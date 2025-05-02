`timescale 1ns / 1ps
//////////////////////////////////////////////////////////////////////////////////
// Company: 
// Engineer: 
// 
// Create Date: 2023/10/24 17:58:40
// Design Name: 
// Module Name: tb_receive_nd
// Project Name: 
// Target Devices: 
// Tool Versions: 
// Description: 
// 
// Dependencies: 
// 
// Revision:
// Revision 0.01 - File Created
// Additional Comments:
// 
//////////////////////////////////////////////////////////////////////////////////

`include "../../sources_1/new/frame_datapath.vh"

module tb_receive_nd #(
  parameter DATA_WIDTH = 64,
  parameter ID_WIDTH = 3
) (

);
  reg [127:0] interface_ip_address [3:0];
  reg [47:0]  interface_mac_address [3:0];
  reg [39:0]  mac_pre;
  logic reset;
  
  frame_beat in8, in, out;
  initial begin
    mac_pre = 40'h1069641F8C;
    interface_mac_address[0] = {8'h10, mac_pre};
    interface_mac_address[1] = {8'h11, mac_pre};
    interface_mac_address[2] = {8'h12, mac_pre};
    interface_mac_address[3] = {8'h13, mac_pre};
    interface_ip_address[0] = {8'h10, mac_pre[39:24], 16'hfeff, mac_pre[23:2], ~mac_pre[1], mac_pre[0], 48'h0, 16'h80fe};
    interface_ip_address[1] = {8'h11, mac_pre[39:24], 16'hfeff, mac_pre[23:2], ~mac_pre[1], mac_pre[0], 48'h0, 16'h80fe};
    interface_ip_address[2] = {8'h12, mac_pre[39:24], 16'hfeff, mac_pre[23:2], ~mac_pre[1], mac_pre[0], 48'h0, 16'h80fe};
    interface_ip_address[3] = {8'h13, mac_pre[39:24], 16'hfeff, mac_pre[23:2], ~mac_pre[1], mac_pre[0], 48'h0, 16'h80fe};
    in8.meta = 0;
    reset = 1;
    #6000
    reset = 0;


  end

  wire clk_125M;

  clock clock_i (
    .clk_125M (clk_125M)
  );

  wire in8_ready, in_ready;
  logic out_ready;

  wire we;
  wire [127:0] write_ip;
  wire [ID_WIDTH - 1:0] write_interface;
  wire [47:0] write_mac;

  axis_model axis_model_i (
    .clk (clk_125M),
    .reset (reset),
    .m_data (in8.data),
    .m_keep (in8.keep),
    .m_last (in8.last),
    .m_user (in8.user),
    .m_id (in8.meta.id),
    .m_valid (in8.valid),
    .m_ready (in8_ready)
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

  receive_nd dut (
    .clk(clk_125M),
    .rst(reset),
    .in(in),
    .in_ready(in_ready),
    .out(out),
    .out_ready(out_ready),
    
    .we(we),
    .write_ip(write_ip),
    .write_interface(write_interface),
    .write_mac(write_mac),

    .interface_ip_address(interface_ip_address),
    .interface_mac_address(interface_mac_address)
  );

  reg out_is_first;
  always @ (posedge clk_125M or posedge reset) begin
    if (reset) begin
      out_is_first <= 1'b1;
    end else begin
      if (out.valid && out_ready) begin
        out_is_first <= out.last;
      end
    end
  end

  reg [ID_WIDTH - 1:0] dest;
  reg drop_by_prev;  // Dropped by the previous frame?
  always @ (posedge clk_125M or posedge reset) begin
    if (reset) begin
      dest <= 0;
      drop_by_prev <= 1'b0;
    end else begin
      if (out_is_first && out.valid && out_ready) begin
        dest <= out.meta.dest;
        drop_by_prev <= out.meta.drop_next;
      end
    end
  end

  // Rewrite dest.
  wire [ID_WIDTH - 1:0] dest_current = out_is_first ? out.meta.dest : dest;

  frame_beat filtered;
  wire filtered_ready;

  frame_filter #(
    .DATA_WIDTH(DATAW_WIDTH),
    .ID_WIDTH(ID_WIDTH)
  ) frame_filter_i(
    .eth_clk(clk_125M),
    .reset(reset),

    .s_data(out.data),
    .s_keep(out.keep),
    .s_last(out.last),
    .s_user(out.user),
    .s_id(dest_current),
    .s_valid(out.valid),
    .s_ready(out_ready),

    .drop(out.meta.drop || drop_by_prev),

    .m_data(filtered.data),
    .m_keep(filtered.keep),
    .m_last(filtered.last),
    .m_user(filtered.user),
    .m_id(filtered.meta.dest),
    .m_valid(filtered.valid),
    .m_ready(filtered_ready)
  );

  // README: Change the width back. You can remove this.
  frame_beat out8;
  logic out8_ready;
  frame_beat_width_converter #(DATAW_WIDTH, DATA_WIDTH) frame_beat_downsizer(
    .clk(clk_125M),
    .rst(reset),

    .in(filtered),
    .in_ready(filtered_ready),
    .out(out8),
    .out_ready(out8_ready)
  );

  axis_receiver axis_receiver_i(
    .clk(clk_125M),
    .reset(reset),

    .s_data(out8.data),
    .s_keep(out8.keep),
    .s_last(out8.last),
    .s_user(out8.user),
    .s_dest(out8.meta.dest),
    .s_valid(out8.valid),
    .s_ready(out8_ready)
  );

endmodule
