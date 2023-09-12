module sensor (
    input clk_uart,
    input rst,
    input wireless_tx,
    output wireless_rx,
    output wireless_set,
    output logic [15:0] acceleration,
    output logic [15:0] direction
);
  // https://wit-motion.yuque.com/wumwnr/ltst03/vl3tpy
  parameter HEADER = 8'h55;
  parameter MESSAGE_LENGTH = 11;
  parameter ACCELERATION_TYPE = 8'h51;
  parameter ACCERATION_INDEX = 0; // X axis
  parameter DIRECTION_TYPE = 8'h53;
  parameter DIRECTION_INDEX = 1; // Y axis

  wire [7:0] data;
  wire next_byte;
  async_receiver async_receiver_inst (
      .clk_uart(clk_uart),
      .rst(rst),
      .rx(wireless_tx),  // rx is connected to tx
      .next_byte(next_byte),
      .data(data)
  );

  logic [ 3:0] byte_index;  // index of the next byte to read
  logic [ 7:0] message_type;
  logic [ 7:0] checksum;
  logic [15:0] acceleration_buffer;
  logic [15:0] direction_buffer;

  always_ff @(posedge clk_uart) begin
    if (rst) begin
      byte_index <= 0;
      acceleration <= 0;
      direction <= 0;
    end else if (next_byte) begin
      if (byte_index == 0) begin  // read header
        if (data == HEADER) begin
          byte_index <= 1;
        end
      end else if (byte_index == 1) begin
        byte_index <= 2;
        message_type <= data;
        checksum <= HEADER + data;
      end else if (byte_index == MESSAGE_LENGTH - 1) begin  // read checksum and update outputs
        byte_index <= 0;
        if (checksum == data) begin
          if (message_type == ACCELERATION_TYPE) acceleration <= acceleration_buffer;
          if (message_type == DIRECTION_TYPE) direction <= direction_buffer;
        end
      end else begin  // read data
        byte_index <= byte_index + 1;
        checksum   <= checksum + data;
        if (message_type == ACCELERATION_TYPE) begin
          if (byte_index == ACCERATION_INDEX * 2 + 2) acceleration_buffer[7:0] <= data;
          if (byte_index == ACCERATION_INDEX * 2 + 3) acceleration_buffer[15:8] <= data;
        end
        if (message_type == DIRECTION_TYPE) begin
          if (byte_index == DIRECTION_INDEX * 2 + 2) direction_buffer[7:0] <= data;
          if (byte_index == DIRECTION_INDEX * 2 + 3) direction_buffer[15:8] <= data;
        end
      end
    end
  end

  assign wireless_rx  = 1;  // idle
  assign wireless_set = 1;  // set to HIGH to avoid entering AT command mode
endmodule
