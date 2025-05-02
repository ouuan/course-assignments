`timescale 1ns / 1ps

`include "frame_datapath.vh"

// Handle ND cache not found, replace the packet by NS
//  set meta.last and meta.drop_next
module send_multicast_ns (
    input wire clk,
    input wire rst,

    input frame_beat in,
    output wire in_ready,

    output frame_beat out,
    input wire out_ready,

    input wire [127:0] interface_ip_address [IF_NUM-1:0],
    input wire [47:0]  interface_mac_address [IF_NUM-1:0]
);

    frame_beat s1;
    wire s1_ready;
    assign in_ready = !in.valid || s1_ready;

    always_ff @ (posedge clk or posedge rst) begin
        if (rst) begin
            s1 <= 0;
        end else if (s1_ready) begin
            s1 <= in;
            if (`should_handle_mac(in)) begin
                if (in.meta.nd_cache_not_found) begin
                    s1.keep <= {{(DATAW_WIDTH / 8 - TOTAL_LENGTH_NS){1'b0}}, {(TOTAL_LENGTH_NS){1'b1}}};

                    if (!in.last) begin
                      s1.last <= 1;
                      s1.meta.drop_next <= 1;
                    end

                    s1.data.src <= interface_mac_address[in.meta.dest];
                    s1.data.dst <= {in.meta.next_hop[127:104], 8'hff, 16'h3333};

                    s1.data.ip6.src <= interface_ip_address[in.meta.dest];
                    s1.data.ip6.dst <= {in.meta.next_hop[127:104], 8'hff, 16'h0100, 80'h02ff};

                    s1.data.ip6.next_hdr <= NEXTHDR_ICMP6;

                    s1.data.ip6.payload_len <= {ICMP_PAYLOAD_LENGTH_NS, 8'b0};
                    s1.data.ip6.hop_limit <= 8'd255;

                    s1.data.ip6.p.icmp6.icmp6_type <= ICMPTYPE_NS;
                    s1.data.ip6.p.icmp6.code <= 8'b0;

                    s1.data.ip6.p.icmp6.checksum <= 16'b0;
                    s1.meta.write_checksum <= 1'b1;

                    s1.data.ip6.p.icmp6.nd.ns.reserved <= 32'b0;
                    s1.data.ip6.p.icmp6.nd.ns.target_address <= in.meta.next_hop;
                    s1.data.ip6.p.icmp6.nd.ns.option_type <= NDOPT_SOURCEMAC;
                    s1.data.ip6.p.icmp6.nd.ns.option_length <= 8'd1;
                    s1.data.ip6.p.icmp6.nd.ns.option_data <= interface_mac_address[in.meta.dest];
                end
            end
        end
    end

    assign s1_ready = out_ready;
    assign out = s1;

endmodule
