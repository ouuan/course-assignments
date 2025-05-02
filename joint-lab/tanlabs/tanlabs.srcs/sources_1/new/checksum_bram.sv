`timescale 1ns / 1ps

module checksum_bram #(
    ADDR_WIDTH = 9
) (
    input wire clk,

    input wire [ADDR_WIDTH-1:0] raddr,
    output wire [31:0] rdata,

    input wire [ADDR_WIDTH-1:0] waddr,
    input wire [31:0] wdata,
    input wire [3:0] we,

    output wire [15:0] checksum
);
  logic [31:0] checksum_reg = 0;
  logic [31:0] checksum_acc;
  logic [3:0] we_prev = 0;
  logic [ADDR_WIDTH-1:0] waddr_prev = 0;
  logic [31:0] wdata_prev = 0;
  wire [31:0] wdata_old;

  always_comb begin : CALC
    checksum_acc = checksum_reg;

    if (waddr_prev < 5) disable CALC;

    for (integer i = 0; i < 4; i += 1) begin
      static logic [15:0] tmpa, tmpb;
      if (we_prev[i] == 0 || (waddr_prev == 5 && i == 3)) continue;
      tmpa = 16'(wdata_prev[8*i+:8]);
      tmpb = 16'(wdata_old[8*i+:8]);
      if (i % 2 == 1 || (waddr_prev == 5 && i == 2)) begin
        tmpa = tmpa << 8;
        tmpb = tmpb << 8;
      end
      checksum_acc = checksum_acc + tmpa + 16'(~tmpb);
    end

    checksum_acc = checksum_acc[15:0] + checksum_acc[31:16];
  end

  always_ff @(posedge clk) begin
    checksum_reg <= checksum_acc;
    we_prev <= we;
    waddr_prev <= waddr;
    wdata_prev <= wdata;
  end

  bram_checksum bram (
      .clka (clk),
      .ena  (we != 0),
      .wea  (we),
      .addra(waddr),
      .dina (wdata),
      .douta(wdata_old),
      .clkb (clk),
      .web  (0),
      .addrb(raddr),
      .dinb (0),
      .doutb(rdata)
  );

  assign checksum = checksum_reg[15:0] + checksum_reg[31:16];
endmodule
