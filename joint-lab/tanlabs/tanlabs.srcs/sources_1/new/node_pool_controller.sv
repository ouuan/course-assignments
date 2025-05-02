`timescale 1ns / 1ps
`include "frame_datapath.vh"

module node_pool_controller (
    input wire clk,
    input wire rst,

    input wire wb_cyc_i,
    input wire wb_stb_i,
    output logic wb_ack_o,
    input wire [31:0] wb_adr_i,
    input wire [31:0] wb_dat_i,
    output logic [31:0] wb_dat_o,
    input wire [3:0] wb_sel_i,
    input wire wb_we_i,

    output logic [NODE_ADDR_WIDTH-1:0] bram_node_addr[STAGE_NUM:1],
    input bitmap_node bram_node_rdata[STAGE_NUM:1],
    output bitmap_node bram_node_wdata[STAGE_NUM:1],
    output logic bram_node_we[STAGE_NUM:1]
);
  logic [NODE_POOL_STAGE_WIDTH-1:0] addr_comb;
  logic [STAGE_WIDTH-1:0] stage_comb;
  logic [1:0] part_comb;
  logic addr_changed_comb;
  logic [2:0] written_next_comb;

  bitmap_node rdata_reg;
  bitmap_node wdata_reg;
  logic [2:0] written_reg;
  logic [STAGE_WIDTH-1:0] stage_reg;
  logic [NODE_POOL_STAGE_WIDTH-1:0] addr_reg;

  enum logic [1:0] {
    IDLE,
    READ1,
    READ2,
    WRITE
  } state;

  always_comb begin
    part_comb = wb_adr_i[3:2];
    addr_comb = wb_adr_i[4+:NODE_POOL_STAGE_WIDTH];
    stage_comb = wb_adr_i[4+NODE_POOL_STAGE_WIDTH+:STAGE_WIDTH];

    addr_changed_comb = stage_reg != stage_comb || addr_reg != addr_comb;
    written_next_comb = addr_changed_comb ? 0 : written_reg;
    if (wb_we_i) written_next_comb |= 1 << part_comb;

    for (integer i = 1; i <= STAGE_NUM; ++i) begin
      bram_node_addr[i] = 0;
      bram_node_we[i] = 0;
      bram_node_wdata[i] = 0;
    end

    bram_node_addr[stage_reg+1] = {1'b0, addr_reg};
    bram_node_we[stage_reg+1] = state == WRITE;
    bram_node_wdata[stage_reg+1] = wdata_reg;

    wb_ack_o = state == IDLE && wb_cyc_i && wb_stb_i && (wb_we_i || !addr_changed_comb);

    case (part_comb)
      0: wb_dat_o = {rdata_reg.is_leaf, rdata_reg.entry_mask, rdata_reg.child_mask};
      1: wb_dat_o = 32'(rdata_reg.child_addr);
      2: wb_dat_o = 32'(rdata_reg.entry_addr);
      default: wb_dat_o = 0;
    endcase
  end

  always_ff @(posedge clk or posedge rst) begin
    if (rst) begin
      state <= IDLE;
      rdata_reg <= 0;
      wdata_reg <= 0;
      written_reg <= 0;
      stage_reg <= 0;
      addr_reg <= 0;
    end else begin
      case (state)
        IDLE:
        if (wb_stb_i && wb_cyc_i) begin
          if (addr_changed_comb) begin
            if (!wb_we_i) state <= READ1;
            stage_reg <= stage_comb;
            addr_reg  <= addr_comb;
          end

          written_reg <= written_next_comb;

          if (wb_we_i) begin
            case (part_comb)
              0: begin
                wdata_reg.is_leaf <= wb_dat_i[31];
                wdata_reg.entry_mask <= wb_dat_i[30:16];
                wdata_reg.child_mask <= wb_dat_i[15:0];
              end
              1: wdata_reg.child_addr <= wb_dat_i[NODE_ADDR_WIDTH-1:0];
              2: wdata_reg.entry_addr <= wb_dat_i[NODE_ADDR_WIDTH-1:0];
            endcase

            if (written_next_comb == 3'b111) state <= WRITE;
          end
        end
        READ1:   state <= READ2;
        READ2: begin
          rdata_reg <= bram_node_rdata[stage_reg+1];
          state <= IDLE;
        end
        WRITE: begin
          state <= IDLE;
          rdata_reg <= wdata_reg;
          written_reg <= 0;
        end
        default: state <= IDLE;
      endcase
    end
  end
endmodule
