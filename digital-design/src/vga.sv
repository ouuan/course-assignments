// http://tinyvga.com/vga-timing/1280x800@60Hz
module vga #(
    COOR_WIDTH = 12,
    HSIZE = 1280,
    HFP = 1344,
    HSP = 1480,
    HMAX = 1680,
    VSIZE = 800,
    VFP = 801,
    VSP = 804,
    VMAX = 828,
    HSPP = 1,
    VSPP = 1,
    RAM_WIDTH = 20,
    FRAME_LEFT = 0,
    FRAME_RIGHT = 1280,
    FRAME_TOP = 250,
    FRAME_BOTTOM = 550,
    RAM_SIZE = (FRAME_RIGHT - FRAME_LEFT) * (FRAME_BOTTOM - FRAME_TOP),
    BUFFER_WIDTH = 4,
    NIGHT_RATE_WIDTH = 6,
    MAX_NIGHT_RATE = (1 << NIGHT_RATE_WIDTH) - 1
) (
    input clk_33m,
    input clk_vga,
    // write inputs are in clk 33m
    input [COOR_WIDTH-1:0] write_x,  // [0, 1280)
    input [COOR_WIDTH-1:0] write_y,  // [0, 300)
    input [2:0] write_palette,
    input [NIGHT_RATE_WIDTH-1:0] night_rate,
    output rst_screen_vga,
    output rst_screen_33m,  // refresh screen, swap RAM r/w parts
    output logic hsync,
    output logic vsync,
    output logic data_enable,
    output logic [7:0] output_red,
    output logic [7:0] output_green,
    output logic [7:0] output_blue
);
  logic [COOR_WIDTH-1:0] buffer_x[BUFFER_WIDTH-1:0];
  logic [COOR_WIDTH-1:0] buffer_y[BUFFER_WIDTH-1:0];

  wire  [COOR_WIDTH-1:0] read_x;
  wire  [COOR_WIDTH-1:0] read_y;
  assign read_x = buffer_x[0];
  assign read_y = buffer_y[0];

  wire [COOR_WIDTH-1:0] output_x;
  wire [COOR_WIDTH-1:0] output_y;
  assign output_x = buffer_x[BUFFER_WIDTH-1];
  assign output_y = buffer_y[BUFFER_WIDTH-1];

  // iterate coordinates & store in the ring buffer
  always_ff @(posedge clk_vga) begin
    if (buffer_x[0] == HMAX - 1) begin
      buffer_x[0] <= 0;
      if (buffer_y[0] == VMAX - 1) buffer_y[0] <= 0;
      else buffer_y[0] <= buffer_y[0] + 1;
    end else begin
      buffer_x[0] <= buffer_x[0] + 1;
    end
    for (integer i = 1; i < BUFFER_WIDTH; i += 1) begin
      buffer_x[i] <= buffer_x[i-1];
      buffer_y[i] <= buffer_y[i-1];
    end
  end

  // rst_screen
  assign rst_screen_vga = read_y == VSIZE && read_x < 16;
  ram_cross_domain cross_domain_rst_screen (
      .wrclock(clk_vga),
      .wraddress(0),
      .data(rst_screen_vga),
      .wren(1),
      .rdclock(clk_33m),
      .rdaddress(0),
      .q(rst_screen_33m)
  );

  // VGA sync and enable signals
  always_ff @(posedge clk_vga) begin
    hsync <= ((output_x >= HFP) && (output_x < HSP)) ? HSPP : !HSPP;
    vsync <= ((output_y >= VFP) && (output_y < VSP)) ? VSPP : !VSPP;
    data_enable <= ((output_x < HSIZE) & (output_y < VSIZE));
  end

  logic [NIGHT_RATE_WIDTH-1:0] night_rate_buffer;

  // partition of RAM
  logic read_part = 0;
  always_ff @(posedge clk_vga) begin
    if (read_y == VSIZE && read_x == 8) begin
      read_part <= ~read_part;
      night_rate_buffer <= night_rate;
    end
  end
  wire write_part;
  ram_cross_domain cross_domain_write_part (
      .wrclock(clk_vga),
      .wraddress(0),
      .data(~read_part),
      .wren(1),
      .rdclock(clk_33m),
      .rdaddress(0),
      .q(write_part)
  );

  // write RAM
  wire [RAM_WIDTH-1:0] write_addr;
  assign write_addr = write_part * RAM_SIZE + write_x + write_y * HSIZE;
  wire write_enable;
  assign write_enable = !rst_screen_33m && write_palette != 0;

  // read RAM
  wire [RAM_WIDTH-1:0] read_addr;
  wire read_enable;
  assign read_enable = read_x >= FRAME_LEFT && read_x < FRAME_RIGHT && read_y >= FRAME_TOP && read_y < FRAME_BOTTOM;
  assign read_addr = read_part * RAM_SIZE + (read_x - FRAME_LEFT) + (read_y - FRAME_TOP) * (FRAME_RIGHT - FRAME_LEFT);

  wire [2:0] read_palette;

  // RAM
  ram_vga ram_vga_inst (
      .rdclock(clk_vga),
      .rdaddress(read_addr),
      .q(read_palette),
      .rden(read_enable),
      .wrclock(clk_33m),
      .wraddress(write_addr),
      .data(write_palette),
      .wren(write_enable)
  );

  // get color from palette
  wire [7:0] read_red  [0:MAX_NIGHT_RATE];
  wire [7:0] read_green[0:MAX_NIGHT_RATE];
  wire [7:0] read_blue [0:MAX_NIGHT_RATE];
  wire [7:0] bg_red [0:MAX_NIGHT_RATE];
  wire [7:0] bg_green [0:MAX_NIGHT_RATE];
  wire [7:0] bg_blue [0:MAX_NIGHT_RATE];

  genvar i;
  generate
    for (i = 0; i <= MAX_NIGHT_RATE; i += 1) begin : palette_gen
      palette #(
          .MAX_NIGHT_RATE(MAX_NIGHT_RATE),
          .NIGHT_RATE(i)
      ) palette_read (
          .palette_index(read_palette),
          .red(read_red[i]),
          .green(read_green[i]),
          .blue(read_blue[i])
      );

      palette #(
          .MAX_NIGHT_RATE(MAX_NIGHT_RATE),
          .NIGHT_RATE(i)
      ) palette_bg (
          .palette_index(7),
          .red(bg_red[i]),
          .green(bg_green[i]),
          .blue(bg_blue[i])
      );
    end
  endgenerate

  // color outputs
  always_ff @(posedge clk_vga) begin
    if (output_x >= FRAME_LEFT && output_x < FRAME_RIGHT && output_y >= FRAME_TOP && output_y < FRAME_BOTTOM) begin
      output_red   <= read_red[night_rate_buffer];
      output_green <= read_green[night_rate_buffer];
      output_blue  <= read_blue[night_rate_buffer];
    end else if (output_x < HSIZE && output_y < VSIZE) begin
      output_red   <= bg_red[night_rate_buffer];
      output_green <= bg_green[night_rate_buffer];
      output_blue  <= bg_blue[night_rate_buffer];
    end else begin
      output_red   <= 0;
      output_green <= 0;
      output_blue  <= 0;
    end
  end
endmodule
