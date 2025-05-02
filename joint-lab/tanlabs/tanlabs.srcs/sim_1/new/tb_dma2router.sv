`timescale 1ns / 1ps

module tb_dma2router;
  wire clk;
  clock clock_i (.clk_125M(clk));
  logic rst;

  wire [8:0] router_raddr;
  logic [31:0] router_rdata;
  wire router_read_finished;
  logic [10:0] router_rlen;
  wire [7:0] internal_tx_data;
  wire internal_tx_last;
  wire internal_tx_user;
  wire internal_tx_valid;

  initial begin
    router_rlen = 20;
    router_rdata = 0;
    rst = 1;
    #8
    rst = 0;
    #8
    router_rdata = 32'h11223344;
    #8
    router_rdata = 32'h11223344;
    #8
    router_rdata = 32'haabbccdd;

  end

  dma2router dut (
      .clk,
      .rst,
      .router_raddr,
      .router_rdata,
      .router_read_finished,
      .router_rlen,
      .internal_tx_data,
      .internal_tx_last,
      .internal_tx_user,
      .internal_tx_valid
  );
endmodule
