`timescale 1ns / 1ps

module tb_router2dma_sram;
  wire clk;
  clock clock_i (.clk_125M(clk));
  logic rst;

  logic [31:0] wb_dat_i;
  logic wb_ack_i;
  wire wb_cyc_o;
  wire wb_stb_o;
  wire [31:0] wb_adr_o;
  wire [31:0] wb_dat_o;
  wire [3:0] wb_sel_o;
  wire wb_we_o;

  logic [7:0] internal_rx_data;
  logic internal_rx_last;
  logic internal_rx_user;
  logic internal_rx_valid;

  wire internal_rx_ready;

  logic [7:0] data[];

  initial begin
    data = '{8'h33,8'h33,8'hff,8'h69,8'h10,8'h24,8'h8c,8'h1f,8'h64,8'h69,8'h10,8'h01,8'h86,8'hdd,8'h60,8'h00,8'h00,8'h00,8'h00,8'h20,8'h3a,8'hff,8'h2a,8'h0e,8'haa,8'h06,8'h04,8'h97,8'h00,8'h00,8'h00,8'h00,8'h00,8'h00,8'h00,8'h00,8'h00,8'h02,8'hff,8'h02,8'h00,8'h00,8'h00,8'h00,8'h00,8'h00,8'h00,8'h00,8'h00,8'h01,8'hff,8'h69,8'h10,8'h24,8'h87,8'h00,8'h8f,8'hac,8'h00,8'h00,8'h00,8'h00,8'hfe,8'h80,8'h00,8'h00,8'h00,8'h00,8'h00,8'h00,8'h8e,8'h1f,8'h64,8'hff,8'hfe,8'h69,8'h10,8'h24,8'h01,8'h01,8'h8c,8'h1f,8'h64,8'h69,8'h10,8'h01};
    wb_ack_i = 1;
    wb_dat_i = 0;
    internal_rx_last = 0;
    internal_rx_user = 0;
    internal_rx_valid = 0;
    rst = 1;
    #8
    rst = 0;

    #8
    for (integer i = 0; i < 86; i = i + 1) begin
      #8
      internal_rx_valid = 0;
      #8
      internal_rx_valid = 1;
      internal_rx_data = data[i];
    end
    internal_rx_last = 1;
    #8
    internal_rx_last = 0;
    internal_rx_valid = 0;

    #8000
    for (integer i = 0; i < 86; i = i + 1) begin
      #8
      internal_rx_valid = 0;
      #8
      internal_rx_valid = 1;
      internal_rx_data = data[i];
    end
    internal_rx_last = 1;
    #8
    internal_rx_last = 0;
    internal_rx_valid = 0;

    #8000
    for (integer i = 0; i < 2000; i = i + 1) begin
      #8
      internal_rx_valid = 0;
      #8
      internal_rx_valid = 1;
      internal_rx_data = i;
    end
    internal_rx_last = 1;
    #8
    internal_rx_last = 0;
    internal_rx_valid = 0;

    #8000
    for (integer i = 0; i < 86; i = i + 1) begin
      #8
      internal_rx_valid = 0;
      #8
      internal_rx_valid = 1;
      internal_rx_data = data[i];
    end
    internal_rx_last = 1;
    #8
    internal_rx_last = 0;
    internal_rx_valid = 0;

    #8000
    for (integer i = 0; i < 86; i = i + 1) begin
      #8
      internal_rx_valid = 0;
      #8
      internal_rx_valid = 1;
      internal_rx_data = i;
    end
    internal_rx_last = 1;
    #8
    internal_rx_last = 0;
    internal_rx_valid = 0;

    #8000
    for (integer i = 0; i < 86; i = i + 1) begin
      #8
      internal_rx_valid = 0;
      #8
      internal_rx_valid = 1;
      internal_rx_data = data[i];
    end
    internal_rx_last = 1;
    #8
    internal_rx_last = 0;
    internal_rx_valid = 0;

  end

  router2dma_sram dut (
      .clk,
      .rst,

      .wb_dat_i,
      .wb_ack_i,
      .wb_cyc_o,
      .wb_adr_o,
      .wb_dat_o,
      .wb_sel_o,
      .wb_stb_o,
      .wb_we_o,

      .internal_rx_data,
      .internal_rx_last,
      .internal_rx_user,
      .internal_rx_valid,
      .internal_rx_ready
  );
endmodule
