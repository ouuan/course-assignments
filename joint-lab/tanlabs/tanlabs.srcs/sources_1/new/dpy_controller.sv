`timescale 1ns / 1ps

`include "cpu/rv32i_cpu.vh"

module dpy_controller #(
    DATA_WIDTH = 32,
    ADDR_WIDTH = 32
) (
    input wire clk,
    input wire rst,

    output logic [7:0] dpy0,
    output logic [7:0] dpy1,

    input wire wb_cyc_i,
    input wire wb_stb_i,
    output wire wb_ack_o,
    input wire [ADDR_WIDTH-1:0] wb_adr_i,
    input wire [DATA_WIDTH-1:0] wb_dat_i,
    output wire [DATA_WIDTH-1:0] wb_dat_o,
    input wire [DATA_WIDTH/8-1:0] wb_sel_i,
    input wire wb_we_i
);

  logic [31:0] value;
  logic [31:0] counter;
  logic [ 2:0] seconds;

  assign wb_ack_o = wb_cyc_i && wb_stb_i;
  assign wb_dat_o = value;

  always_ff @(posedge clk or posedge rst) begin
    if (rst) begin
      value   <= 0;
      counter <= 0;
      seconds <= 0;
    end else begin
      if (wb_cyc_i && wb_stb_i && wb_we_i) begin
        for (integer i = 0; i < 4; ++i) begin
          if (wb_sel_i[i]) begin
            value[i*8+:8] <= wb_dat_i[i*8+:8];
          end
        end
      end
      if (counter < SYS_CLOCK_FREQ) counter <= counter + 1;
      else begin
        counter <= 0;
        seconds <= seconds + 1;
      end
    end
  end

  logic [3:0] dig0, dig1;
  wire [7:0] seg0, seg1;

  always_comb begin
    case (seconds)
      3'd0: begin
        dig1 = value[31:28];
        dig0 = value[27:24];
      end
      3'd1: begin
        dig1 = value[23:20];
        dig0 = value[19:16];
      end
      3'd2: begin
        dig1 = value[15:12];
        dig0 = value[11:8];
      end
      default: begin
        dig1 = value[7:4];
        dig0 = value[3:0];
      end
    endcase

    if (seconds < 4 && counter < SYS_CLOCK_FREQ / 4) begin
      dpy0 = 0;
      dpy1 = 0;
    end else begin
      dpy0 = seg0;
      dpy1 = seg1;
    end
  end

  SEG7_LUT seg7_lut_0 (
      .iDIG (dig0),
      .oSEG1(seg0)
  );
  SEG7_LUT seg7_lut_1 (
      .iDIG (dig1),
      .oSEG1(seg1)
  );

endmodule
