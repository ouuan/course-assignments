`timescale 1ps / 1ps

`include "../../sources_1/new/frame_datapath.vh"

module tb_internet_checksum #(
    parameter DATA_WIDTH = 64,
    parameter ID_WIDTH   = 3
);
  logic reset;
  initial begin
    reset = 1;
    #6000 reset = 0;
  end

  wire clk_125M;

  clock clock_i (.clk_125M);

  frame_beat in8, in, out;
  wire in8_ready, in_ready;
  logic out_ready;
  wire [15:0] checksum;

  axis_model axis_model_i (
      .clk(clk_125M),
      .reset,
      .m_data(in8.data),
      .m_keep(in8.keep),
      .m_last(in8.last),
      .m_user(in8.user),
      .m_id(in8.meta.id),
      .m_valid(in8.valid),
      .m_ready(in8_ready)
  );

  always @(posedge clk_125M or posedge reset) begin
    if (reset) begin
      in8.is_first <= 1'b1;
    end else begin
      if (in8.valid && in8_ready) begin
        in8.is_first <= in8.last;
      end
    end
  end

  frame_beat_width_converter #(DATA_WIDTH, DATAW_WIDTH) frame_beat_upsizer (
      .clk(clk_125M),
      .rst(reset),
      .in(in8),
      .in_ready(in8_ready),
      .out(in),
      .out_ready(in_ready)
  );

  internet_checksum dut (
      .clk(clk_125M),
      .rst(reset),
      .in,
      .in_ready,
      .out,
      .out_ready,
      .checksum
  );

  assign out_ready = 1;
endmodule
