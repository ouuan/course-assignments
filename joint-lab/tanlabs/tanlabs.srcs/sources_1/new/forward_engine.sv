`timescale 1ns / 1ps

`include "frame_datapath.vh"

module forward_engine (
    input wire clk,
    input wire rst,

    input frame_beat in,
    output wire in_ready,

    output frame_beat out,
    input wire out_ready,

    input wire [127:0] interface_ip_address [IF_NUM-1:0],
    input wire [47:0]  interface_mac_address [IF_NUM-1:0],

    // nd cache read signal
    output wire [127:0] nd_cache_read_ip,
    output wire [ID_WIDTH - 1:0] nd_cache_read_interface,
    input wire [47:0] nd_cache_read_mac,
    input wire nd_cache_read_exist,

    output wire [NODE_ADDR_WIDTH-1:0] bram_node_addr[STAGE_NUM:1],
    input bitmap_node bram_node_data[STAGE_NUM:1],
    output wire [NODE_ADDR_WIDTH-1:0] bram_entry_addr,
    input wire [ENTRY_WIDTH-1:0] bram_entry_data,
    output logic [ENTRY_WIDTH-1:0] next_hop_addr,
    input next_hop_t next_hop_data
);

    // State 2: Drop frame that hop limit <= 1, and subtract by 1
    frame_beat s2;
    wire s2_ready;
    assign in_ready = !in.valid || s2_ready;

    always_ff @ (posedge clk or posedge rst) begin
        if (rst) begin
            s2 <= 0;
        end else begin
            if (s2_ready) begin
                s2 <= in;
                if (`should_handle(in)) begin
                    if (in.data.ip6.dst == interface_ip_address[in.meta.id] || in.data.ip6.dst == RIPNG_MULTICAST_IP) begin
                        s2.meta.dont_touch <= 1;
                        s2.meta.dest <= ID_CPU;
                        s2.data.dst[47:40] <= 8'(in.meta.id);
                    end else if (// The unspecified address must not be used as the destination address
                        // of IPv6 packets or in IPv6 Routing headers.  An IPv6 packet with a
                        // source address of unspecified must never be forwarded by an IPv6 router.
                        in.data.ip6.src == 128'h0 || in.data.ip6.dst == 128'h0 ||
                        // The loopback address must not be used as the source address in IPv6
                        // packets that are sent outside of a single node.  An IPv6 packet with
                        // a destination address of loopback must never be sent outside of a
                        // single node and must never be forwarded by an IPv6 router.
                        in.data.ip6.src == {8'h1, 120'h0} || in.data.ip6.dst == {8'h1, 120'h0} ||
                        // Multicast addresses must not be used as source addresses in IPv6
                        // packets or appear in any Routing header.
                        in.data.ip6.src[7:0] == 8'hff ||
                        // Routers must not forward any multicast packets beyond of the scope
                        // indicated by the scop field in the destination multicast address.
                        (in.data.ip6.dst[7:0] == 8'hff && in.data.ip6.dst[15:12] <= 2) ||
                        in.data.ip6.hop_limit <= 1
                    ) begin
                        s2.meta.drop <= 1;
                    end else begin
                        s2.data.ip6.hop_limit <= in.data.ip6.hop_limit - 1;
                        if (
                            // Routers must not forward any packets with Link-Local source or
                            // destination addresses to other links.
                            in.data.ip6.src[63:0] == 64'h80fe || in.data.ip6.dst[63:0] == 64'h80fe
                        ) begin
                            s2.meta.next_hop_from_dst <= 1;
                            s2.meta.dest <= in.meta.id;
                            s2.meta.dont_touch_ip <= 1;
                        end
                    end
                end
            end
        end
    end


    // State 3: Query forward table to fetch egress interface and egress IP address based on target IP
    //  s3.meta.drop = 1 if not found
    frame_beat s3;
    wire s3_ready;

    forwarding_table_pipeline forwarding_table_i (
        .clk(clk),
        .rst(rst),
        .in(s2),
        .in_ready(s2_ready),
        .out(s3),
        .out_ready(s3_ready),
        .bram_node_addr,
        .bram_node_data,
        .bram_entry_addr,
        .bram_entry_data,
        .next_hop_addr,
        .next_hop_data
    );

    // State 4: Query neighbor cache to fetch next hop MAC address based on next hop IPv6 address
    frame_beat s4;
    wire s4_ready;
    assign s3_ready = !s3.valid || s4_ready;
    assign nd_cache_read_ip = s3.meta.next_hop;
    assign nd_cache_read_interface = s3.meta.dest;

    always_ff @ (posedge clk or posedge rst) begin
        if (rst) begin
            s4 <= 0;
        end else if (s4_ready) begin
            s4 <= s3;
            if (`should_handle_mac(s3)) begin
                if (nd_cache_read_exist) begin
                    s4.data.dst <= nd_cache_read_mac;
                    s4.data.src <= interface_mac_address[s3.meta.dest];
                end else begin
                    s4.meta.nd_cache_not_found <= 1;
                end
            end
        end
    end

    // State 5: Handle ND cache not found, replace the packet by NS
    //  set last and meta.drop_next
    frame_beat s5;
    wire s5_ready;

    send_multicast_ns send_multicast_ns_i (
        .clk(clk),
        .rst(rst),
        .in(s4),
        .in_ready(s4_ready),
        .out(s5),
        .out_ready(s5_ready),
        .interface_ip_address(interface_ip_address),
        .interface_mac_address(interface_mac_address)
    );

    frame_beat s6;
    wire [15:0] checksum;

    always_comb begin
        out = s6;
        if (out.meta.write_checksum) begin
            out.data.ip6.p.icmp6.checksum = ~checksum;
            out.meta.write_checksum = 1'b0;
        end
    end

    internet_checksum internet_checksum_out (
        .clk(clk),
        .rst(rst),
        .in(s5),
        .in_ready(s5_ready),
        .out(s6),
        .out_ready(out_ready),
        .checksum
    );

endmodule

