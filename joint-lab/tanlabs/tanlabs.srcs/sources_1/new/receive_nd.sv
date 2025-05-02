`timescale 1ns / 1ps

`include "frame_datapath.vh"

module receive_nd #(
    parameter DATA_WIDTH = 64,
    parameter ID_WIDTH   = 3
) (
    input wire clk,
    input wire rst,

    // AXI stream in
    input frame_beat in,
    output wire in_ready,

    // AXI stream out
    output frame_beat out,
    input wire out_ready,

    // nd cache write signal
    output logic we,
    output logic [127:0] write_ip,
    output logic [ID_WIDTH - 1:0] write_interface,
    output logic [47:0] write_mac,

    // interface address
    input wire [127:0] interface_ip_address [IF_NUM-1:0],
    input wire [ 47:0] interface_mac_address[IF_NUM-1:0],
    input wire [127:0] solicited_ip_address [IF_NUM-1:0],
    input wire [ 47:0] solicited_mac_address[IF_NUM-1:0]
);

  frame_beat checksum_out;
  logic [15:0] checksum_result;
  logic checksum_out_ready;

  internet_checksum internet_checksum_in (
      .clk(clk),
      .rst(rst),

      .in(in),
      .in_ready(in_ready),

      .out(checksum_out),
      .out_ready(checksum_out_ready),

      .checksum(checksum_result)
  );

  frame_beat s1;
  logic s1_ready;
  assign checksum_out_ready = !checksum_out.valid || s1_ready;

  always_ff @(posedge clk or posedge rst) begin
    if (rst) begin
      s1 <= 0;
      we <= 0;
      write_ip <= 0;
      write_interface <= 0;
      write_mac <= 0;
    end else if (s1_ready) begin
      s1 <= checksum_out;
      we <= 1'b0;
      if (`should_handle(checksum_out)) begin
        if (checksum_out.meta.id == ID_CPU) begin
          for (integer i = 0; i < 4; ++i) begin
            if (checksum_out.data.src == interface_mac_address[i]) begin
              s1.meta.dont_touch <= 1'b1;
              s1.meta.dest <= i[2:0];
            end
          end
        end else if ((checksum_out.last && (checksum_out.keep & {54{1'b1}} != {54{1'b1}})) ||
            checksum_out.data.ethertype != ETHERTYPE_IP6 ||
            checksum_out.data.ip6.version != 4'd6 ||
            !(checksum_out.data.dst == interface_mac_address[checksum_out.meta.id] ||
              checksum_out.data.dst == solicited_mac_address[checksum_out.meta.id] ||
              checksum_out.data.dst == 48'hff_ff_ff_ff_ff_ff ||
              checksum_out.data.dst == RIPNG_MULTICAST_MAC))
          s1.meta.drop <= 1'b1;  // invalid IPv6 datagram or we are not the recipient on the link layer
        else if (checksum_out.data.ip6.next_hdr == NEXTHDR_ICMP6 &&
                 (checksum_out.data.ip6.p.icmp6.icmp6_type >= ICMPTYPE_ND_MIN &&
                  checksum_out.data.ip6.p.icmp6.icmp6_type <= ICMPTYPE_ND_MAX)) begin // handle ND message
          s1.meta.dont_touch <= 1'b1;  // don't forward
          if (!checksum_out.last ||
              checksum_out.data.ip6.hop_limit != 8'hff ||
              checksum_result != 16'hffff ||
              checksum_out.data.ip6.p.icmp6.code != 8'd0 ||
              // only accept 0 or 1 option with length 1
              !((checksum_out.data.ip6.payload_len == {8'd24, 8'd0} && checksum_out.keep == {78{1'b1}}) ||
                (checksum_out.data.ip6.payload_len == {8'd32, 8'd0} && checksum_out.keep == {86{1'b1}} &&
                 checksum_out.data.ip6.p.icmp6.nd.ns.option_length == 8'd1)) ||
              !(checksum_out.data.ip6.dst == interface_ip_address[checksum_out.meta.id] ||
                checksum_out.data.ip6.dst == solicited_ip_address[checksum_out.meta.id]))
            s1.meta.drop <= 1'b1;  // invalid ND message or we are not the recipient on the IP layer
          else if (checksum_out.data.ip6.p.icmp6.icmp6_type == ICMPTYPE_NS) begin  // NS message
            if (checksum_out.data.ip6.src == 128'd0 ||  // DAD
                checksum_out.data.ip6.p.icmp6.nd.ns.target_address != interface_ip_address[checksum_out.meta.id])
              s1.meta.drop <= 1'b1;
            else begin
              s1.keep <= {86{1'b1}};
              s1.data.ip6.payload_len <= {8'd32, 8'd0};
              s1.meta.dest <= checksum_out.meta.id;
              s1.data.src <= interface_mac_address[checksum_out.meta.id];

              s1.data.ip6.src <= interface_ip_address[checksum_out.meta.id];
              s1.data.ip6.dst <= checksum_out.data.ip6.src;

              s1.data.ip6.p.icmp6.icmp6_type <= ICMPTYPE_NA;
              s1.data.ip6.p.icmp6.code <= 8'b0;
              s1.data.ip6.p.icmp6.checksum <= 16'b0;
              s1.meta.write_checksum <= 1'b1;

              s1.data.ip6.p.icmp6.nd.na.override_flag <= 1'b1;
              s1.data.ip6.p.icmp6.nd.na.router_flag <= 1'b1;
              s1.data.ip6.p.icmp6.nd.na.solicited_flag <= 1'b1;

              s1.data.ip6.p.icmp6.nd.na.option_type <= NDOPT_TARGETMAC;
              s1.data.ip6.p.icmp6.nd.na.option_length <= 8'd1;
              s1.data.ip6.p.icmp6.nd.na.option_data <= interface_mac_address[checksum_out.meta.id];

              if (checksum_out.data.ip6.payload_len == {8'd32, 8'd0}) begin  // has option
                if (checksum_out.data.ip6.p.icmp6.nd.ns.option_type != NDOPT_SOURCEMAC)
                  s1.meta.drop <= 1'b1;
                else begin
                  s1.data.dst <= checksum_out.data.ip6.p.icmp6.nd.ns.option_data;
                  // update ND cache
                  we <= 1'b1;
                  write_ip <= checksum_out.data.ip6.src;
                  write_interface <= checksum_out.meta.id;
                  write_mac <= checksum_out.data.ip6.p.icmp6.nd.ns.option_data;
                end
              end else begin  // no option
                if (checksum_out.data.ip6.dst != interface_ip_address[checksum_out.meta.id])
                  s1.meta.drop <= 1'b1;  // AR without source MAC
                else begin  // NUD without source MAC
                  // let forward engine to resolve the MAC address
                  s1.meta.dont_touch <= 1'b0;
                  s1.meta.dont_touch_ip <= 1'b1;
                  s1.meta.next_hop_from_src <= 1'b1;
                end
              end
            end
          end else if (checksum_out.data.ip6.p.icmp6.icmp6_type == ICMPTYPE_NA) begin  // NA message
            if (checksum_out.data.ip6.dst[7:0] == 8'hff ||  // multicast IP destination address
                checksum_out.data.ip6.p.icmp6.nd.na.target_address[7:0] == 8'hff ||  // multicast target address
                checksum_out.data.ip6.p.icmp6.nd.na.solicited_flag == 1'b0 ||  // unsolicited NA
                checksum_out.data.ip6.payload_len != {8'd32, 8'd0} ||  // no option
                checksum_out.data.ip6.p.icmp6.nd.na.option_type != NDOPT_TARGETMAC) // no target MAC address
              s1.meta.drop <= 1'b1;
            else begin
              // update nd-cache
              // only a simple cache, ignore the override flag
              we <= 1'b1;
              write_ip <= checksum_out.data.ip6.p.icmp6.nd.na.target_address;
              write_mac <= checksum_out.data.ip6.p.icmp6.nd.na.option_data;
              write_interface <= checksum_out.meta.id;
              s1.meta.drop <= 1'b1;  // drop NA message
            end
          end else s1.meta.drop <= 1'b1;  // ND, but not NS or NA
        end
      end
    end
  end

  assign out = s1;
  assign s1_ready = out_ready;

endmodule
