`timescale 1ns / 1ps `default_nettype none

`include "frame_datapath.vh"

// calculate the Internet checksum of an IP datagram
module internet_checksum #(
    CHECKSUM_WIDTH = 16
) (
    input wire clk,
    input wire rst,
    input frame_beat in,
    output wire in_ready,
    output frame_beat out,  // copied from input, synced with the checksum
    input wire out_ready,
    // The checksum output is already in big endian.
    // It is only valid in the last beat.
    output wire [CHECKSUM_WIDTH-1:0] checksum
);
  logic state;
  frame_beat buffer;  // used for output
  logic [CHECKSUM_WIDTH*2-1:0] sum;

  assign in_ready = state == 0 && out_ready;
  assign checksum = sum[CHECKSUM_WIDTH-1:0];

  logic [DATAW_WIDTH-1:0] pseudo_header;  // valid for first beat only

  always_comb begin
    static ip6_hdr ip6;
    ip6 = in.data.ip6;
    pseudo_header[111:0] = 112'd0;  // Ethernet segment
    pseudo_header[239:112] = ip6.src;
    pseudo_header[367:240] = ip6.dst;
    pseudo_header[399:368] = {ip6.payload_len, 16'd0};
    pseudo_header[431:400] = {ip6.next_hdr, 24'd0};
    // This is technically not a part of the pseudo header, but
    pseudo_header[DATAW_WIDTH-1:432] = ip6.p;
  end

  always_ff @(posedge clk or posedge rst) begin
    if (rst) begin
      state <= 0;
      out <= 0;
      sum <= 0;
      buffer <= 0;
    end else if (out_ready) begin
      if (state == 0) begin
        out <= 0;
        if (in.valid) begin
          // the data to calculate checksum from
          static logic [DATAW_WIDTH-1:0] data;
          // temporary variable to sum up the 16-bit parts
          static logic [CHECKSUM_WIDTH*2-1:0] acc;

          if (in.last || !out_ready) begin
            state  <= 1;
            buffer <= in;
          end else begin
            // no need to add carry out into checksum if not last beat
            state <= 0;
            out   <= in;
          end

          if (in.is_first) begin
            data = pseudo_header;
            acc  = 0;
          end else begin
            data = in.data;
            acc  = sum;
          end

          if (in.last) begin
            for (integer i = 0; i < DATAW_WIDTH; i = i + 8) begin
              if (!in.keep[i/8]) data[i+:8] = 8'd0;
            end
          end

          for (integer i = 0; i < DATAW_WIDTH; i = i + CHECKSUM_WIDTH) begin
            acc = acc + data[i+:CHECKSUM_WIDTH];
          end
          sum <= acc;
        end
      end else begin
        static logic [31:0] tmp;
        state <= 0;
        out   <= buffer;
        tmp = sum[CHECKSUM_WIDTH*2-1:CHECKSUM_WIDTH] + sum[CHECKSUM_WIDTH-1:0];
        sum <= tmp[CHECKSUM_WIDTH*2-1:CHECKSUM_WIDTH] + tmp[CHECKSUM_WIDTH-1:0];
      end
    end
  end
endmodule
