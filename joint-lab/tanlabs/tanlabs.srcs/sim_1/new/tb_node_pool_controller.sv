`timescale 1ns / 1ps

`include "../../sources_1/new/frame_datapath.vh"

module tb_node_pool_controller;
  wire  clk;
  logic rst;

  clock clock_i (.clk_125M(clk));

  logic wb_cyc_i = 0;
  logic wb_stb_i = 0;
  wire wb_ack_o;
  logic [31:0] wb_adr_i = 0;
  logic [31:0] wb_dat_i = 0;
  wire [31:0] wb_dat_o;
  logic [3:0] wb_sel_i = 4'b1111;
  logic wb_we_i = 0;

  wire [NODE_ADDR_WIDTH-1:0] bram_node_addr[STAGE_NUM:1];
  bitmap_node bram_node_rdata[STAGE_NUM:1];
  bitmap_node bram_node_wdata[STAGE_NUM:1];
  wire bram_node_we[STAGE_NUM:1];

  logic [STAGE_WIDTH-1:0] read_stage;
  logic [NODE_POOL_STAGE_WIDTH-1:0] read_addr;
  bitmap_node read_dat;

  always_ff @(posedge clk) begin
    bram_node_rdata <= '{default: 'x};
    if (bram_node_addr[read_stage+1] == {1'b0, read_addr})
      bram_node_rdata[read_stage+1] <= read_dat;
  end

  task wb_read(input logic [31:0] adr);
    #1;
    wb_cyc_i = 1;
    wb_stb_i = 1;
    wb_adr_i = adr;
    wb_dat_i = 0;
    wb_we_i  = 0;
    #7;
    while (!wb_ack_o) #8;
    #1;
    wb_stb_i = 0;
    wb_cyc_i = 0;
    #7;
  endtask

  task wb_write(input logic [31:0] adr, input logic [31:0] dat);
    #1;
    wb_cyc_i = 1;
    wb_stb_i = 1;
    wb_adr_i = adr;
    wb_dat_i = dat;
    wb_we_i  = 1;
    #7;
    while (!wb_ack_o) #8;
    #1;
    wb_stb_i = 0;
    wb_cyc_i = 0;
    #7;
  endtask

  task read(input logic [STAGE_WIDTH-1:0] stage, input logic [NODE_POOL_STAGE_WIDTH-1:0] addr,
            input bitmap_node dat);
    read_stage = stage;
    read_addr  = addr;
    read_dat   = dat;
    wb_read(32'({stage, addr, 2'd0, 2'd0}));
    wb_read(32'({stage, addr, 2'd1, 2'd0}));
    wb_read(32'({stage, addr, 2'd2, 2'd0}));
  endtask

  task write(input logic [STAGE_WIDTH-1:0] stage, input logic [NODE_POOL_STAGE_WIDTH-1:0] addr,
             input bitmap_node dat);
    wb_write(32'({stage, addr, 2'd0, 2'd0}), {dat.is_leaf, dat.entry_mask, dat.child_mask});
    wb_write(32'({stage, addr, 2'd1, 2'd0}), 32'(dat.child_addr));
    wb_write(32'({stage, addr, 2'd2, 2'd0}), 32'(dat.entry_addr));
  endtask

  initial begin
    rst = 1;
    #50;
    rst = 0;
    #50;
    read(0, 2, '{is_leaf: 1, entry_mask: 1, child_mask: 1, child_addr: 1, entry_addr: 1});
    read(1, 2, '{is_leaf: 0, entry_mask: 2, child_mask: 2, child_addr: 2, entry_addr: 2});
    write(1, 2, '{is_leaf: 1, entry_mask: 3, child_mask: 3, child_addr: 3, entry_addr: 3});
    read(1, 3, '{is_leaf: 0, entry_mask: 4, child_mask: 4, child_addr: 4, entry_addr: 4});
    read(1, 3, '{is_leaf: 0, entry_mask: 4, child_mask: 4, child_addr: 4, entry_addr: 4});
    write(0, 2, '{is_leaf: 1, entry_mask: 5, child_mask: 5, child_addr: 5, entry_addr: 5});
    read(0, 2, '{is_leaf: 1, entry_mask: 5, child_mask: 5, child_addr: 5, entry_addr: 5});
    $finish;
  end

  node_pool_controller dut (
      .clk(clk),
      .rst(rst),

      .wb_cyc_i(wb_cyc_i),
      .wb_stb_i(wb_stb_i),
      .wb_ack_o(wb_ack_o),
      .wb_adr_i(wb_adr_i),
      .wb_dat_i(wb_dat_i),
      .wb_dat_o(wb_dat_o),
      .wb_sel_i(wb_sel_i),
      .wb_we_i (wb_we_i),

      .bram_node_addr(bram_node_addr),
      .bram_node_rdata(bram_node_rdata),
      .bram_node_wdata(bram_node_wdata),
      .bram_node_we(bram_node_we)
  );
endmodule
