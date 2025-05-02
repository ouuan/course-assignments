module sram_controller #(
    parameter DATA_WIDTH = 32,
    parameter ADDR_WIDTH = 32,

    parameter SRAM_ADDR_WIDTH = 20,
    parameter SRAM_DATA_WIDTH = 32,

    localparam SRAM_BYTES = SRAM_DATA_WIDTH / 8,
    localparam SRAM_BYTE_WIDTH = $clog2(SRAM_BYTES)
) (
    // clk and reset
    input wire clk_i,
    input wire rst_i,

    // wishbone slave interface
    input wire wb_cyc_i,
    input wire wb_stb_i,
    output reg wb_ack_o,
    input wire [ADDR_WIDTH-1:0] wb_adr_i,
    input wire [DATA_WIDTH-1:0] wb_dat_i,
    output reg [DATA_WIDTH-1:0] wb_dat_o,
    input wire [DATA_WIDTH/8-1:0] wb_sel_i,
    input wire wb_we_i,

    // sram interface
    output reg [SRAM_ADDR_WIDTH-1:0] sram_addr,
    inout wire [SRAM_DATA_WIDTH-1:0] sram_data,
    output reg sram_ce_n,
    output reg sram_oe_n,
    output reg sram_we_n,
    output reg [SRAM_BYTES-1:0] sram_be_n
);

  wire [SRAM_DATA_WIDTH-1:0] sram_data_i;
  logic [SRAM_DATA_WIDTH-1:0] sram_data_o;
  logic sram_data_t;
  assign sram_data   = sram_data_t ? 'bz : sram_data_o;
  assign sram_data_i = sram_data;

  enum logic [2:0] {
    INIT,
    READ1,
    READ2,
    READ3,
    WRITE1,
    WRITE2
  } state;

  logic [SRAM_ADDR_WIDTH-1:0] addr_reg;
  logic [SRAM_DATA_WIDTH-1:0] data_reg;
  logic [SRAM_BYTES-1:0] be_n_reg;

  always_comb begin
    wb_ack_o = state == READ3 || (state == INIT && wb_cyc_i && wb_stb_i && wb_we_i);
    wb_dat_o = sram_data_i;
    sram_addr = wb_adr_i[2+:SRAM_ADDR_WIDTH];
    sram_data_t = !wb_we_i;
    sram_data_o = wb_dat_i;
    sram_ce_n = state == INIT && !wb_stb_i;
    sram_oe_n = wb_we_i;
    sram_we_n = state != WRITE1;
    sram_be_n = ~wb_sel_i;

    if (state == WRITE1 || state == WRITE2) begin
      sram_addr   = addr_reg;
      sram_data_t = 0;
      sram_data_o = data_reg;
      sram_oe_n   = 1;
      sram_be_n   = be_n_reg;
    end
  end

  always_ff @(posedge clk_i or posedge rst_i) begin
    if (rst_i) begin
      state <= INIT;
      addr_reg <= 0;
      data_reg <= 0;
      be_n_reg <= 0;
    end else
      case (state)
        INIT: begin
          if (wb_cyc_i && wb_stb_i) begin
            if (wb_we_i) begin
              state <= WRITE1;
              addr_reg <= sram_addr;
              data_reg <= sram_data_o;
              be_n_reg <= sram_be_n;
            end else state <= READ1;
          end
        end
        READ1: state <= READ2;
        READ2: state <= READ3;
        READ3: state <= INIT;
        WRITE1: state <= WRITE2;
        WRITE2: state <= INIT;
        default: state <= INIT;
      endcase
  end

endmodule
