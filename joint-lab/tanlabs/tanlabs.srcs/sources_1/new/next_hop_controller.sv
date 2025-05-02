`timescale 1ns / 1ps
`include "frame_datapath.vh"

module next_hop_controller (
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

    output logic [ENTRY_WIDTH-1:0] next_hop_addr,
    input next_hop_t next_hop_rdata,
    output next_hop_t next_hop_wdata,
    output logic next_hop_we
);
  logic [ENTRY_WIDTH-1:0] addr_comb;
  logic [2:0] part_comb;
  logic addr_changed_comb;
  logic [4:0] written_next_comb;

  next_hop_t wdata_reg;
  logic [4:0] written_reg;
  logic [ENTRY_WIDTH-1:0] addr_reg;

  enum logic [1:0] {
    IDLE,
    READ,
    WRITE
  } state;

  always_comb begin
    part_comb = wb_adr_i[4:2];
    addr_comb = wb_adr_i[5+:ENTRY_WIDTH];

    addr_changed_comb = addr_reg != addr_comb;
    written_next_comb = addr_changed_comb ? 0 : written_reg;
    if (wb_we_i) written_next_comb |= 1 << part_comb;

    next_hop_addr = addr_reg;
    next_hop_we = state == WRITE;
    next_hop_wdata = wdata_reg;

    wb_ack_o = state == IDLE && wb_cyc_i && wb_stb_i && (wb_we_i || !addr_changed_comb);

    case (part_comb)
      0: wb_dat_o = next_hop_rdata.next_hop[0+:32];
      1: wb_dat_o = next_hop_rdata.next_hop[32+:32];
      2: wb_dat_o = next_hop_rdata.next_hop[64+:32];
      3: wb_dat_o = next_hop_rdata.next_hop[96+:32];
      4: wb_dat_o = 32'({next_hop_rdata.direct, next_hop_rdata.dest});
      default: wb_dat_o = 0;
    endcase
  end

  always_ff @(posedge clk or posedge rst) begin
    if (rst) begin
      state <= IDLE;
      wdata_reg <= 0;
      written_reg <= 0;
      addr_reg <= 0;
    end else begin
      case (state)
        IDLE:
        if (wb_stb_i && wb_cyc_i) begin
          if (addr_changed_comb) begin
            if (!wb_we_i) state <= READ;
            addr_reg <= addr_comb;
          end

          written_reg <= written_next_comb;

          if (wb_we_i) begin
            case (part_comb)
              0: wdata_reg.next_hop[0+:32] <= wb_dat_i;
              1: wdata_reg.next_hop[32+:32] <= wb_dat_i;
              2: wdata_reg.next_hop[64+:32] <= wb_dat_i;
              3: wdata_reg.next_hop[96+:32] <= wb_dat_i;
              4: begin
                wdata_reg.direct <= wb_dat_i[2];
                wdata_reg.dest   <= wb_dat_i[1:0];
              end
            endcase

            if (written_next_comb == 5'b11111) state <= WRITE;
          end
        end
        READ: state <= IDLE;
        WRITE: begin
          state <= IDLE;
          written_reg <= 0;
        end
        default: state <= IDLE;
      endcase
    end
  end
endmodule
