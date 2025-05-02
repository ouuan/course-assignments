`timescale 1ns / 1ps

module tb_checksum_bram;
  wire clk;
  clock clock_i (.clk_125M(clk));

  logic [9:0] raddr;
  wire [31:0] rdata;
  logic [9:0] waddr;
  logic [31:0] wdata;
  logic [3:0] we;
  wire [15:0] checksum;

  initial begin
    raddr = 0;
    waddr = 0;
    wdata = 0;
    we = 0;

    #5;

    waddr = 0;
    wdata = 32'hffffxxxx;
    we = 4'b1100;
    #8;

    waddr = 1;
    wdata = 32'hffffffff;
    we = 4'b1111;
    #8;

    waddr = 2;
    wdata = 32'h69641f8c;
    we = 4'b1111;
    #8;

    waddr = 3;
    wdata = 32'hxxxx0110;
    we = 4'b0011;
    #8;

    waddr = 3;
    wdata = 32'hdd86xxxx;
    we = 4'b1100;
    #8;

    waddr = 4;
    wdata = 32'h00000060;
    we = 4'b1111;
    #8;

    waddr = 5;
    wdata = 32'hxxxx1400;
    we = 4'b0011;
    #8;

    waddr = 5;
    wdata = 32'hxx11xxxx;
    we = 4'b0100;
    #8;

    waddr = 5;
    wdata = 32'h40xxxxxx;
    we = 4'b1000;
    #8;

    waddr = 6;
    wdata = 32'h06aa0e2a;
    we = 4'b1111;
    #8;

    waddr = 7;
    wdata = 32'h00009704;
    #8;

    waddr = 8;
    wdata = 32'h0;
    #8;

    waddr = 9;
    wdata = 32'h02000000;
    #8;

    waddr = 10;
    wdata = 32'h00f00224;
    #8;

    waddr = 11;
    wdata = 32'h0;
    #8;

    waddr = 12;
    #8;

    waddr = 13;
    wdata = 32'h01000000;
    #8;

    waddr = 14;
    wdata = 32'hxxxx0700;
    we = 4'b0011;
    #8;

    waddr = 14;
    wdata = 32'h0700xxxx;
    we = 4'b1100;
    #8;

    waddr = 15;
    wdata = 32'hxxxx1400;
    we = 4'b0011;
    #8;

    waddr = 15;
    wdata = 32'h774exxxx;
    we = 4'b1100;
    #16;

    waddr = 16;
    wdata = 32'h6c6c6568;
    we = 4'b1111;
    #8;

    waddr = 17;
    wdata = 32'h30202c6f;
    #8;

    waddr = 18;
    wdata = 32'h31303030;
    #8;

    for (integer i = 0; i < 5; ++i) begin
      $write("%03h:", i * 16);
      for (integer j = 0; j < 4; ++j) begin
        raddr = i * 4 + j;
        #8;
        for (integer k = 0; k < 4; ++k)
          $write(" %02h", rdata[k*8+:8]);
      end
      $write("\n");
    end

    waddr = 15;
    wdata = 32'h0000xxxx;
    we = 4'b1100;
    #16;

    we = 0;
    #8;
    $finish;
  end

  checksum_bram dut (
      .clk,
      .raddr,
      .rdata,
      .waddr,
      .wdata,
      .we,
      .checksum
  );
endmodule
