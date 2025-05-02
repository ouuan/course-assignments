`timescale 1ns / 1ps
`include "frame_datapath.vh"

module entry_pool_controller (
    input wire clk,
    input wire rst,

    input wire wb_cyc_i,
    input wire wb_stb_i,
    output logic wb_ack_o,
    input wire [31:0] wb_adr_i,
    input wire [31:0] wb_dat_i,
    output logic [31:0] wb_dat_o,
    input wire [3:0] wb_sel_i,
    input wire wb_we_i,

    output logic [NODE_ADDR_WIDTH-1:0] bram_entry_addr,
    input wire [ENTRY_WIDTH-1:0] bram_entry_rdata,
    output logic [ENTRY_WIDTH-1:0] bram_entry_wdata,
    output logic bram_entry_we
);
  enum logic {
    INIT,
    OUTPUT
  } state;

  always_comb begin
    wb_ack_o = state == OUTPUT || (state == INIT && wb_stb_i && wb_cyc_i && wb_we_i);
    wb_dat_o = bram_entry_rdata << (wb_adr_i[1:0] * 8);
    bram_entry_addr = wb_cyc_i && wb_stb_i ? wb_adr_i[NODE_ADDR_WIDTH-1:0] : 0;
    bram_entry_wdata = wb_dat_i[wb_adr_i[1:0]*8+:ENTRY_WIDTH];
    bram_entry_we = wb_cyc_i && wb_stb_i && wb_we_i;
  end

  always_ff @(posedge clk or posedge rst) begin
    if (rst) begin
      state <= INIT;
    end else begin
      case (state)
        INIT:
        if (wb_cyc_i && wb_stb_i && !wb_we_i) begin
          state <= OUTPUT;
        end
        OUTPUT: state <= INIT;
      endcase
    end
  end
endmodule
