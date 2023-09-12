`timescale 1ns / 1ps

module paint_background #(
    COOR_WIDTH = 12,
    WIDTH = 1280,
    HEIGHT = 300
) (
    input clk_33m,
    input rst,  // start painting
    // write to RAM
    output [COOR_WIDTH-1:0] write_x,
    output [COOR_WIDTH-1:0] write_y,
    output [2:0] write_palette,
    // paint finished
    output finished
);
  logic [COOR_WIDTH-1:0] x;
  logic [COOR_WIDTH-1:0] y;

  assign finished = y == HEIGHT;

  always_ff @(posedge clk_33m) begin
    if (rst) begin
      x <= 0;
      y <= 0;
    end else if (!finished) begin
      if (x == WIDTH - 1) begin
        x <= 0;
        y <= y + 1;
      end else begin
        x <= x + 1;
      end
    end
  end

  assign write_x = finished ? 0 : x;
  assign write_y = finished ? 0 : y;
  assign write_palette = finished ? 0 : 7;  // 7 is white
endmodule
