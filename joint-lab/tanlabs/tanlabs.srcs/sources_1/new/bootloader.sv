`timescale 1ns / 1ps
`include "cpu/rv32i_cpu.vh"

module bootloader #(
    DATA_WIDTH = 32,
    ADDR_WIDTH = 32,
    SRAM_ADDR_WIDTH = 21,
    INST_ADDR_WIDTH = 13,
    TEXT_SIZE = 32'h6000
) (
    input wire clk,
    input wire rst,

    output logic wb_cyc_o,
    output logic wb_stb_o,
    input wire wb_ack_i,
    output logic [ADDR_WIDTH-1:0] wb_adr_o,
    output logic [DATA_WIDTH-1:0] wb_dat_o,
    input wire [DATA_WIDTH-1:0] wb_dat_i,
    output logic [DATA_WIDTH/8-1:0] wb_sel_o,
    output logic wb_we_o,

    output logic inst_we,
    output logic [INST_ADDR_WIDTH-1:0] inst_addr,
    output logic [31:0] inst_wdata,

    output logic [47:0] mac_address[3:0],
    output logic [127:0] ip_address[3:0],
    output logic [127:0] solicited_ip_address[3:0],
    output logic [47:0] solicited_mac_address[3:0],

    output logic finished
);

  enum logic [2:0] {
    READ_TEXT,
    READ_MAC,
    READ_IP,
    RESET_DMA,
    FINISHED
  } state;

  logic [INST_ADDR_WIDTH-1:0] text_pos;
  logic [2:0] mac_pos;
  logic [3:0] ip_pos;
  logic [8:0] dma_pos;

  always_comb begin
    wb_cyc_o = wb_stb_o;
    wb_dat_o = 0;
    wb_sel_o = 4'b1111;
    wb_we_o  = 0;

    case (state)
      READ_TEXT: wb_adr_o = SRAM_BASE | (text_pos << 2);
      READ_MAC:  wb_adr_o = SRAM_BASE + TEXT_SIZE + mac_pos * 4;
      READ_IP:   wb_adr_o = SRAM_BASE + TEXT_SIZE + 24 + ip_pos * 4;
      RESET_DMA: begin
        wb_adr_o = ROUTER2CPU_BASE + dma_pos * ROUTER2CPU_PACKET_SIZE;
        wb_we_o  = 1;
      end
      default:   wb_adr_o = 0;
    endcase

    inst_we = state == READ_TEXT && wb_ack_i;
    inst_addr = text_pos;
    inst_wdata = wb_dat_i;

    finished = state == FINISHED;

    for (integer i = 0; i < 4; i += 1) begin
      // https://datatracker.ietf.org/doc/html/rfc4291#page-16
      solicited_ip_address[i]  = {ip_address[i][127:104], 8'hff, 16'h0100, 80'h02ff};

      // https://datatracker.ietf.org/doc/html/rfc2464#section-7
      solicited_mac_address[i] = {solicited_ip_address[i][127:96], 16'h3333};
    end
  end

  always_ff @(posedge clk or posedge rst) begin
    if (rst) begin
      state <= READ_TEXT;
      wb_stb_o <= 1;
      text_pos <= 0;
      mac_pos <= 0;
      ip_pos <= 0;
      dma_pos <= 0;
      mac_address <= '{default: 0};
      ip_address <= '{default: 0};
    end else begin
      if (wb_stb_o) begin
        if (wb_ack_i) begin
          wb_stb_o <= 0;
          case (state)
            READ_MAC:
            case (mac_pos)
              0: mac_address[0][31:0] <= wb_dat_i;
              1: begin
                mac_address[0][47:32] <= wb_dat_i[15:0];
                mac_address[1][15:0]  <= wb_dat_i[31:16];
              end
              2: mac_address[1][47:16] <= wb_dat_i;
              3: mac_address[2][31:0] <= wb_dat_i;
              4: begin
                mac_address[2][47:32] <= wb_dat_i[15:0];
                mac_address[3][15:0]  <= wb_dat_i[31:16];
              end
              5: mac_address[3][47:16] <= wb_dat_i;
            endcase
            READ_IP: ip_address[ip_pos/4][ip_pos%4*32+:32] <= wb_dat_i;
          endcase
        end
      end else begin
        wb_stb_o <= 1;
        case (state)
          READ_TEXT: begin
            if (text_pos == TEXT_SIZE / 4 - 1) state <= READ_MAC;
            else text_pos <= text_pos + 1;
          end
          READ_MAC: begin
            if (mac_pos == 5) state <= READ_IP;
            else mac_pos <= mac_pos + 1;
          end
          READ_IP: begin
            if (ip_pos == 15) state <= RESET_DMA;
            else ip_pos <= ip_pos + 1;
          end
          RESET_DMA: begin
            if (dma_pos == ROUTER2CPU_PACKET_COUNT - 1) begin
              state <= FINISHED;
              wb_stb_o <= 0;
            end else dma_pos <= dma_pos + 1;
          end
          FINISHED: wb_stb_o <= 0;
        endcase
      end
    end
  end

endmodule
