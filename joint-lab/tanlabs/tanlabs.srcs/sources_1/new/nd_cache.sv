`timescale 1ns / 1ps

module nd_cache #(
  parameter CACHE_NUM = 16,
  parameter ID_WIDTH = 3
) (
  input wire eth_clk,
  input wire reset,

  input wire [127:0] write_ip,
  input wire [47:0]  write_mac,
  input wire [ID_WIDTH - 1:0] write_interface,
  input wire         we,
  output wire        write_exist,

  input wire [127:0] read_ip,
  input wire [ID_WIDTH - 1:0] read_interface,
  output wire[47:0]  read_mac,
  output wire        read_exist
);

  localparam CACHE_NUM_WIDTH = $clog2(CACHE_NUM);

  reg [127:0] ip_regs  [CACHE_NUM - 1:0];
  reg [ID_WIDTH - 1:0] interface_regs [CACHE_NUM - 1:0];
  reg [47:0]  mac_regs [CACHE_NUM - 1:0];

  reg [CACHE_NUM_WIDTH - 1:0] count_reg;

  logic [CACHE_NUM - 1:0] is_same_read;
  logic [CACHE_NUM - 1:0] is_same_write;
  logic [47:0] read_mac_comb;

  assign read_exist  = |is_same_read;
  assign write_exist = |is_same_write;

  assign read_mac = (we && read_ip == write_ip) ? write_mac : read_mac_comb;

  always @ (*) begin

    for (integer i = 0; i < CACHE_NUM; i++) begin

      if (read_ip == ip_regs[i] && read_interface == interface_regs[i]) begin
        is_same_read[i] = 1'b1;
      end else begin
        is_same_read[i] = 1'b0; 
      end

      if (write_ip == ip_regs[i] && write_interface == interface_regs[i]) begin
        is_same_write[i] = 1'b1;
      end else begin
        is_same_write[i] = 1'b0;
      end
    end

  end

  always @ (*) begin
    read_mac_comb = 48'b0;
    for (integer i = 0; i < CACHE_NUM; i++) begin
      if (is_same_read[i]) begin
        read_mac_comb = mac_regs[i];
      end
    end
  end

  always @ (posedge eth_clk or posedge reset) begin

    if (reset) begin

      for (integer i = 0; i < CACHE_NUM; i++) begin
        ip_regs[i] <= 128'b0;
        mac_regs[i] <= 48'b0;
        interface_regs[i] <= #ID_WIDTH'b0;
      end
      count_reg <= #CACHE_NUM_WIDTH'b0;

    end else if (we) begin  
      if (write_exist) begin
        for (integer i = 0; i < CACHE_NUM; i++) begin
          if (is_same_write[i]) begin
            mac_regs[i] <= write_mac;
          end
        end
      end else begin
        mac_regs[count_reg] <= write_mac;
        interface_regs[count_reg] <= write_interface;
        ip_regs[count_reg]  <= write_ip;
        count_reg <= count_reg + 1;
      end
    end

  end 
  
  
endmodule
