`timescale 1ns / 1ps

module cpu2router_controller #(
    DATA_WIDTH = 32,
    ADDR_WIDTH = 32,
    MAX_PACKET_LEN = 1516,
    PACKET_LEN_WIDTH = $clog2(MAX_PACKET_LEN),
    PACKET_ADDR_WIDTH = $clog2(MAX_PACKET_LEN * 8 / ADDR_WIDTH)
) (
    input wire clk,
    input wire rst,

    input wire [PACKET_ADDR_WIDTH-1:0] router_raddr,
    output wire [DATA_WIDTH-1:0] router_rdata,
    input wire router_read_finished,
    output wire [PACKET_LEN_WIDTH-1:0] router_rlen,

    input wire wb_cyc_i,
    input wire wb_stb_i,
    output logic wb_ack_o,
    input wire [ADDR_WIDTH-1:0] wb_adr_i,
    input wire [DATA_WIDTH-1:0] wb_dat_i,
    output logic [DATA_WIDTH-1:0] wb_dat_o,
    input wire [DATA_WIDTH/8-1:0] wb_sel_i,
    input wire wb_we_i
);

  logic [PACKET_LEN_WIDTH-1:0] length;
  logic [DATA_WIDTH/8-1:0] cpu_we;
  wire [15:0] checksum;

  always_comb begin
    wb_ack_o = wb_cyc_i && wb_stb_i;
    wb_dat_o = {checksum, 16'(length)};
    cpu_we   = wb_we_i && wb_stb_i && wb_cyc_i && wb_adr_i[16] == 0 ? wb_sel_i : 0;
  end

  always_ff @(posedge clk or posedge rst) begin
    if (rst) begin
      length <= 0;
    end else begin
      if (wb_cyc_i && wb_stb_i && wb_we_i && wb_adr_i[16]) begin
        if (wb_sel_i[0]) length[7:0] <= wb_dat_i[7:0];
        if (wb_sel_i[1]) length[PACKET_LEN_WIDTH-1:8] <= wb_dat_i[PACKET_LEN_WIDTH-1:8];
      end
      if (router_read_finished) length <= 0;
    end
  end

  checksum_bram bram (
      .clk,

      .raddr(router_raddr),
      .rdata(router_rdata),

      .waddr(wb_adr_i[PACKET_ADDR_WIDTH+1:2]),
      .wdata(wb_dat_i),
      .we(cpu_we),

      .checksum
  );

  assign router_rlen = length;

endmodule
