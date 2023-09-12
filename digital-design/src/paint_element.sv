`timescale 1ns / 1ps

module paint_element #(
    COOR_WIDTH = 12,
    ROM_WIDTH = 19,
    SPRITE_WIDTH = 2442,
    FRAME_WIDTH = 1280,
    FRAME_HEIGHT = 300
) (
    input clk_33m,
    input rst,  // start painting
    // left top coordinate on sprite
    input [COOR_WIDTH-1:0] sprite_x,
    input [COOR_WIDTH-1:0] sprite_y,
    // left top coordinate on frame
    input signed [COOR_WIDTH-1:0] frame_x,
    input signed [COOR_WIDTH-1:0] frame_y,
    // size of the element
    input [COOR_WIDTH-1:0] width,
    input [COOR_WIDTH-1:0] height,
    // write to RAM
    output logic [COOR_WIDTH-1:0] write_x,
    output logic [COOR_WIDTH-1:0] write_y,
    output logic [2:0] write_palette,
    // paint finished
    output finished
);

  logic [COOR_WIDTH-1:0] x;
  logic [COOR_WIDTH-1:0] y;
  // ring buffer
  logic [COOR_WIDTH-1:0] last_x[1:0];
  logic [COOR_WIDTH-1:0] last_y[1:0];

  assign finished = last_y[1] == height;

  always_ff @(posedge clk_33m) begin
    if (rst) begin
      x <= 0;
      y <= 0;
      last_x[0] <= 0;
      last_x[1] <= 0;
      last_y[0] <= 0;
      last_y[1] <= 0;
    end else if (!finished) begin
      if (x == width - 1) begin
        x <= 0;
        y <= y + 1;
      end else begin
        x <= x + 1;
      end
      last_x[0] <= x;
      last_x[1] <= last_x[0];
      last_y[0] <= y;
      last_y[1] <= last_y[0];
    end
  end

  logic [ROM_WIDTH-1:0] rom_addr;

  always_comb begin
    if (y == height) rom_addr = 0;
    else rom_addr = (sprite_x + x) + (sprite_y + y) * SPRITE_WIDTH;
  end

  wire [2:0] read_palette;
  rom_sprite rom_sprite_inst (
      .clock(clk_33m),
      .address(rom_addr),
      .q(read_palette)
  );

  always_comb begin
    write_x = 0;
    write_y = 0;
    write_palette = 0;
    if (!finished) begin
      // if inside frame
      if (frame_x + $signed(last_x[1]) >= 0 &&
          frame_x + $signed(last_x[1]) < FRAME_WIDTH &&
          frame_y + $signed(last_y[1]) >= 0 &&
          frame_y + $signed(last_y[1]) < FRAME_HEIGHT) begin
        write_x = frame_x + $signed(last_x[1]);
        write_y = frame_y + $signed(last_y[1]);
        write_palette = read_palette;
      end
    end
  end
endmodule
