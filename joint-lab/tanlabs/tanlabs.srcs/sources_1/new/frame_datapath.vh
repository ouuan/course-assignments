`ifndef _FRAME_DATAPATH_VH_
`define _FRAME_DATAPATH_VH_

// 'w' means wide.
localparam DATAW_WIDTH = 8 * 8 * 11;
localparam ID_WIDTH = 3;

typedef struct packed
{
    logic [(DATAW_WIDTH - 8 * 14 - 8 * 40 - 8 * 32) - 1:0] p;
    logic [47:0] option_data;
    logic [7:0] option_length;
    logic [7:0] option_type;
    logic [127:0] target_address;
    logic [31:0] reserved;
} ns_payload;

typedef struct packed
{
    logic [(DATAW_WIDTH - 8 * 14 - 8 * 40 - 8 * 32) - 1:0] p;
    logic [47:0] option_data;
    logic [7:0] option_length;
    logic [7:0] option_type;
    logic [127:0] target_address;
    logic [23:0] reserved_lo;
    logic router_flag;
    logic solicited_flag;
    logic override_flag;
    logic [4:0]  reserved_hi;
} na_payload;

typedef union packed
{
    ns_payload ns;
    na_payload na;
} nd_payload;

typedef struct packed
{
    nd_payload nd;
    logic [15:0] checksum;
    logic [7:0] code;
    logic [7:0] icmp6_type;
} icmp6_hdr;

typedef struct packed
{
    logic [(DATAW_WIDTH - 8 * 14 - 8 * 40 - 8 * 8 - 8 * 4) - 1:0] p;
    logic [15:0] must_be_zero;
    logic [7:0] version;
    logic [7:0] command;
} ripng_hdr;

typedef struct packed
{
    ripng_hdr ripng;
    logic [15:0] checksum;
    logic [15:0] length;
    logic [15:0] dst_port;
    logic [15:0] src_port;
} udp_hdr;

typedef struct packed
{
    union packed
    {
        icmp6_hdr icmp6;
        udp_hdr udp;
    } p;
    logic [127:0] dst;
    logic [127:0] src;
    logic [7:0] hop_limit;
    logic [7:0] next_hdr;
    logic [15:0] payload_len;
    logic [23:0] flow_lo;
    logic [3:0] version;
    logic [3:0] flow_hi;
} ip6_hdr;

typedef struct packed
{
    ip6_hdr ip6;
    logic [15:0] ethertype;
    logic [47:0] src;
    logic [47:0] dst;
} ether_hdr;

typedef struct packed
{
    // Per-frame metadata.
    // **They are only effective at the first beat.**
    logic [ID_WIDTH - 1:0] id;  // The ingress interface.
    logic [ID_WIDTH - 1:0] dest;  // The egress interface.
    logic drop;  // Drop this frame (i.e., this beat and the following beats till the last)?
    logic dont_touch;  // Do not touch this beat!
    logic dont_touch_ip; // Do not touch the IP layer. Only lookup neighbor cache for MAC address.
    logic write_checksum;
    logic next_hop_from_src; // next_hop is IP src (NA in response to NUD without source MAC)
    logic next_hop_from_dst;
    logic [127:0] next_hop;
    logic nd_cache_not_found;

    // Drop the next frame? It is useful when you need to shrink a frame
    // (e.g., replace an IPv6 packet to an ND solicitation).
    // You can do so by setting both last and drop_next.
    logic drop_next;

} frame_meta;

typedef struct packed
{
    // AXI-Stream signals.
    ether_hdr data;
    logic [DATAW_WIDTH / 8 - 1:0] keep;
    logic last;
    // The IP core will use this "user" signal to indicate errors, so do not modify it!
    logic [DATAW_WIDTH / 8 - 1:0] user;
    logic valid;

    // Handy signals.
    logic is_first;  // Is this the first beat of a frame?

    frame_meta meta;
} frame_beat;

`define should_handle(b) \
(b.valid && b.is_first && !b.meta.drop && !b.meta.dont_touch && !b.meta.dont_touch_ip)
`define should_handle_mac(b) \
(b.valid && b.is_first && !b.meta.drop && !b.meta.dont_touch)

// README: Your code here. You can define some other constants like EtherType.
localparam ID_CPU = 3'd4;  // The interface ID of CPU is 4.
localparam IF_NUM = 4;

localparam ETHERTYPE_IP6 = 16'hdd86;

localparam NEXTHDR_ICMP6 = 8'd58;

localparam ICMPTYPE_ND_MIN = 8'd133;
localparam ICMPTYPE_ND_MAX = 8'd137;
localparam ICMPTYPE_NS = 8'd135;
localparam ICMPTYPE_NA = 8'd136;
localparam ICMP_PAYLOAD_LENGTH_NS = 8'd32; // 1 + 1 + 2 + 4 + 16 + 8 = 32 bytes
localparam TOTAL_LENGTH_NS = 8'd54 + ICMP_PAYLOAD_LENGTH_NS; // 14 + 40 + 32 bytes
localparam ICMP_PAYLOAD_LENGTH_NA = 8'd32;
localparam TOTAL_LENGTH_NA = 8'd54 + ICMP_PAYLOAD_LENGTH_NA;

localparam NDOPT_SOURCEMAC = 8'd1;
localparam NDOPT_TARGETMAC = 8'd2;

localparam RTE_COUNT_WIDTH = 7;

localparam STAGE_NUM = 8;
localparam STAGE_WIDTH = $clog2(STAGE_NUM);
localparam ENTRY_WIDTH = 5;
localparam NODE_ADDR_WIDTH = 18;
localparam NODE_POOL_STAGE_WIDTH = 17;

localparam RIPNG_MULTICAST_IP = {8'h9, 104'h0, 16'h02ff};
localparam RIPNG_MULTICAST_MAC = {8'h9, 24'h0, 16'h3333};

typedef struct packed {
  logic direct;
  logic [1:0] dest;
  logic [127:0] next_hop;
} next_hop_t;

typedef struct packed {
  logic [NODE_ADDR_WIDTH-1:0] entry_addr;
  logic [NODE_ADDR_WIDTH-1:0] child_addr;
  logic is_leaf;
  logic [14:0] entry_mask;
  logic [15:0] child_mask;
} bitmap_node;

`endif
