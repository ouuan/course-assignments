`timescale 1ns / 1ps `default_nettype none
`include "cpu/rv32i_cpu.vh"

module wb_arbiter #(
    MASTER_COUNT = 2
) (
    input wire clk,
    input wire rst,

    input  wbm_signal_o wbm_i[MASTER_COUNT-1:0],
    output wbm_signal_i wbm_o[MASTER_COUNT-1:0],

    input  wbm_signal_i wbs_i,
    output wbm_signal_o wbs_o
);

  localparam MASTER_WIDTH = $clog2(MASTER_COUNT);

  logic [MASTER_WIDTH-1:0] current_master, next_master;

  always_comb begin
    for (integer i = 0; i < MASTER_COUNT; ++i) begin
      wbm_o[i] = 0;
    end

    wbm_o[current_master] = wbs_i;
    wbs_o = wbm_i[current_master];

    next_master = current_master;
    for (integer i = MASTER_COUNT - 1; i >= 0; --i) begin
      if (wbm_i[i].cyc && wbm_i[i].stb) next_master = i;
    end
  end

  always_ff @(posedge clk or posedge rst) begin
    if (rst) current_master <= 0;
    else if (!wbs_o.cyc) current_master <= next_master;
  end
endmodule
