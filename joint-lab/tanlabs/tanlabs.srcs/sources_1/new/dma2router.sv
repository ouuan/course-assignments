`timescale 1ns / 1ps

module dma2router #(
    AXI_WIDTH = 8,
    DATA_WIDTH = 32,
    ADDR_WIDTH = 32,
    MAX_PACKET_LEN = 1516,
    PACKET_LEN_WIDTH = $clog2(MAX_PACKET_LEN),
    PACKET_ADDR_WIDTH = $clog2(MAX_PACKET_LEN * 8 / ADDR_WIDTH)
) (
    input wire clk,
    input wire rst,

    output logic [PACKET_ADDR_WIDTH-1:0] router_raddr,
    input wire [DATA_WIDTH-1:0] router_rdata,
    output logic router_read_finished,
    input wire [PACKET_LEN_WIDTH-1:0] router_rlen,

    output logic [AXI_WIDTH-1:0] internal_tx_data,
    output logic internal_tx_last,
    output logic internal_tx_user,
    output logic internal_tx_valid
);

    assign internal_tx_user = 0;

    reg [PACKET_LEN_WIDTH-1:0] now_len;
    wire [PACKET_LEN_WIDTH-1:0] next_len;
    assign next_len = now_len + 1;
    assign router_raddr = next_len[2+PACKET_ADDR_WIDTH-1:2];

    typedef enum logic { WAIT, READ } state_t;

    state_t state;

    always_ff @(posedge clk) begin
        if (rst) begin
            router_read_finished <= 0;
            internal_tx_data <= 0;
            internal_tx_last <= 0;
            internal_tx_valid <= 0;
            now_len <= 2;
            state <= WAIT;
        end else begin
            if (state == WAIT) begin
                internal_tx_valid <= 0;
                internal_tx_last <= 0;
                router_read_finished <= 0;
                if (!router_read_finished && router_rlen != 0) begin
                    state <= READ;
                end
            end else if (state == READ) begin
                internal_tx_valid <= 1;
                now_len <= next_len;
                // TODO: check endianness
                case (now_len[1:0])
                    2'b00 : internal_tx_data <= router_rdata[7:0];
                    2'b01 : internal_tx_data <= router_rdata[15:8];
                    2'b10 : internal_tx_data <= router_rdata[23:16];
                    2'b11 : internal_tx_data <= router_rdata[31:24];
                    default: internal_tx_data <= 0;
                endcase
                if (next_len == router_rlen) begin
                    internal_tx_last <= 1;
                    state <= WAIT;
                    router_read_finished <= 1;
                    now_len <= 2;
                end
            end
        end
    end

endmodule
