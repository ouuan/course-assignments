module async_receiver (
    input clk_uart,
    input rst,
    input rx,
    output next_byte,
    output logic [7:0] data
);
  parameter OVERSAMPLE_WIDTH = 3;
  parameter OVERSAMPLE_RATE = 1 << OVERSAMPLE_WIDTH;

  wire rx_sync;
  synchronizer #(
      .INITIAL_VALUE(1)
  ) synchronizer_rx (
      .clk(clk_uart),
      .rst(rst),
      .in (rx),
      .out(rx_sync)
  );

  wire rx_filtered;
  spike_filter #(
      .INITIAL_VALUE(1)
  ) spike_filter_rx (
      .clk(clk_uart),
      .rst(rst),
      .in (rx_sync),
      .out(rx_filtered)
  );

  logic [3:0] state;

  logic [OVERSAMPLE_WIDTH-1:0] bit_counter;
  always_ff @(posedge clk_uart) begin
    if (rst || state == 0) bit_counter <= 0;
    else bit_counter <= bit_counter + 1;
  end
  wire next_bit;
  assign next_bit = (bit_counter == OVERSAMPLE_RATE / 2 - 1);  // sample at midpoint

  always @(posedge clk_uart) begin
    if (rst) state <= 0;
    else begin
      case (state)
        4'b0000: if (~rx_filtered) state <= 4'b0001; // start
        // the first next_bit is the midpoint of the starting "0"
        4'b0001: if (next_bit) state <= 4'b1000;
        // ----------- read the byte -----------
        4'b1000: if (next_bit) state <= 4'b1001;
        4'b1001: if (next_bit) state <= 4'b1010;
        4'b1010: if (next_bit) state <= 4'b1011;
        4'b1011: if (next_bit) state <= 4'b1100;
        4'b1100: if (next_bit) state <= 4'b1101;
        4'b1101: if (next_bit) state <= 4'b1110;
        4'b1110: if (next_bit) state <= 4'b1111;
        4'b1111: if (next_bit) state <= 4'b0010;
        // -------------------------------------
        4'b0010: if (next_bit) state <= 4'b0000; // end
        default: state <= 4'b0000;
      endcase
    end
  end

  assign next_byte = (next_bit && state == 4'b0010);

  // shift register to store the byte
  always @(posedge clk_uart) begin
    if (rst) data <= 0;
    else if (state[3] && next_bit) data <= {rx_filtered, data[7:1]};
  end
endmodule
