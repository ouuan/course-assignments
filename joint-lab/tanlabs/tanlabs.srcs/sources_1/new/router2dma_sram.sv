`timescale 1ns / 1ps

`include "frame_datapath.vh"

module router2dma_sram #(
    AXI_WIDTH = 8,
    DATA_WIDTH = 32,
    ADDR_WIDTH = 32,
    MAX_PACKET_LEN = 1516,
    BLOCK_SIZE = 2048,
    BASE_ADDR = 32'h80600000,
    SRAM_ADDR_WIDTH = 18,
    BLOCK_ADDR = BLOCK_SIZE * 8 / DATA_WIDTH,
    BLOCK_ADDR_WIDTH = $clog2(BLOCK_ADDR),
    PACKET_LEN_WIDTH = $clog2(MAX_PACKET_LEN),
    PACKET_ADDR_WIDTH = $clog2(MAX_PACKET_LEN * 8 / ADDR_WIDTH)
) (
    input wire clk,
    input wire rst,

    output logic wb_cyc_o,
    output logic wb_stb_o,
    input wire wb_ack_i,
    output logic [ADDR_WIDTH-1:0] wb_adr_o,
    output logic [DATA_WIDTH-1:0] wb_dat_o,
    input wire [DATA_WIDTH-1:0] wb_dat_i,
    output logic [DATA_WIDTH/8-1:0] wb_sel_o,
    output logic wb_we_o,

    input wire [AXI_WIDTH-1:0] internal_rx_data,
    input wire internal_rx_last,
    input wire internal_rx_user,
    input wire internal_rx_valid,
    output logic internal_rx_ready
);

    reg [SRAM_ADDR_WIDTH-1:0] start_addr;
    reg [1:0] byte_cnt;
    reg [PACKET_LEN_WIDTH-1:0] now_len;

    reg [SRAM_ADDR_WIDTH-1:0] addr;
    assign wb_adr_o = BASE_ADDR | {addr, 2'b0};

    reg [DATA_WIDTH-1:0] checksum;
    wire [DATA_WIDTH-1:0]checksum_tmp;
    wire [DATA_WIDTH-1:0]checksum_result;
    assign checksum_tmp = checksum[15:0] + checksum[31:16];
    assign checksum_result = checksum_tmp[15:0] + checksum_tmp[31:16];

    typedef enum logic[2:0] { READ, WRITE, WRITE_LAST, READ_DONE, WRITE_LEN, WAIT, READ_LEN, DROP } state_t;

    state_t state;

    assign internal_rx_ready = state == READ || state == DROP;

    always_ff @(posedge clk) begin
        if (rst) begin
            start_addr <= 0;
            wb_dat_o <= 0;
            wb_cyc_o <= 0;
            wb_stb_o <= 0;
            wb_we_o <= 0;
            wb_sel_o <= 0;
            addr <= 0;
            byte_cnt <= 2;
            now_len <= 2;
            checksum <= 0;
            state <= READ;
        end else begin
            if (state == READ) begin
                if (internal_rx_valid) begin
                    byte_cnt <= byte_cnt + 1;
                    now_len <= now_len + 1;
                    case (now_len[1:0])
                        2'b00 : wb_dat_o[7:0] <= internal_rx_data;
                        2'b01 : wb_dat_o[15:8] <= internal_rx_data;
                        2'b10 : wb_dat_o[23:16] <= internal_rx_data;
                        2'b11 : wb_dat_o[31:24] <= internal_rx_data;
                        default : wb_dat_o <= 0;
                    endcase
                    // > MTU
                    if (now_len + 1 == MAX_PACKET_LEN && !internal_rx_last) begin
                        state <= DROP;
                    end else if (internal_rx_last) begin
                        wb_cyc_o <= 1'b1;
                        wb_stb_o <= 1'b1;
                        wb_we_o <= 1'b1;
                        wb_sel_o <= 4'b1111;
                        state <= WRITE_LAST;
                    end else if (byte_cnt == 3) begin
                        wb_cyc_o <= 1'b1;
                        wb_stb_o <= 1'b1;
                        wb_we_o <= 1'b1;
                        wb_sel_o <= 4'b1111;
                        state <= WRITE;
                    end
                end
            end else if (state == WRITE) begin
                if (wb_ack_i) begin
                    if (addr[BLOCK_ADDR_WIDTH-1:0] == 5) begin
                        checksum <= checksum + wb_dat_o[15:0] + {wb_dat_o[23:16], 8'b0};
                    end else if (addr[BLOCK_ADDR_WIDTH-1:0] >= 6) begin
                        checksum <= checksum + wb_dat_o[15:0] + wb_dat_o[31:16];
                    end
                    wb_cyc_o <= 0;
                    wb_stb_o <= 0;
                    wb_dat_o <= 0;
                    addr <= addr + 1;
                    state <= READ;
                end
            end else if (state == WRITE_LAST) begin
                if (wb_ack_i) begin
                    checksum <= checksum + wb_dat_o[15:0] + wb_dat_o[31:16];
                    wb_cyc_o <= 0;
                    wb_stb_o <= 0;
                    state <= READ_DONE;
                end
            end else if (state == READ_DONE) begin
                checksum <= 0;
                if (checksum_result[15:0] == 16'hffff) begin
                    wb_dat_o <= now_len;
                    wb_cyc_o <= 1'b1;
                    wb_stb_o <= 1'b1;
                    wb_we_o <= 1'b1;
                    wb_sel_o <= 4'b0011;
                    addr <= start_addr;
                    state <= WRITE_LEN;
                end else begin
                    addr <= start_addr;
                    now_len <= 2;
                    byte_cnt <= 2;
                    state <= READ;
                end
            end else if (state == WRITE_LEN) begin
                if (wb_ack_i) begin
                    wb_cyc_o <= 0;
                    wb_stb_o <= 0;
                    start_addr <= start_addr + BLOCK_ADDR;
                    state <= WAIT;
                end
            end else if (state == WAIT) begin
                addr <= start_addr;
                wb_cyc_o <= 1'b1;
                wb_stb_o <= 1'b1;
                wb_sel_o <= 4'b1111;
                wb_we_o <= 0;
                wb_dat_o <= 0;
                state <= READ_LEN;
            end else if (state == READ_LEN) begin
                if (wb_ack_i) begin
                    wb_cyc_o <= 0;
                    wb_stb_o <= 0;
                    if (wb_dat_i[15:0] == 0) begin
                        now_len <= 2;
                        byte_cnt <= 2;
                        wb_dat_o <= 0;
                        state <= READ;
                    end else begin
                        // SRAM is full, drop it
                        state <= DROP;
                    end
                end
            end else if (state == DROP) begin
                now_len <= 2;
                byte_cnt <= 2;
                wb_cyc_o <= 0;
                wb_stb_o <= 0;
                wb_dat_o <= 0;
                checksum <= 0;
                addr <= start_addr;
                if (internal_rx_valid && internal_rx_last) begin
                    state <= WAIT;
                end
            end
        end
    end

endmodule
