`timescale 1ns / 1ps

`include "frame_datapath.vh"

module forwarding_table_pipeline (
    input wire clk,
    input wire rst,

    input frame_beat in,
    output wire in_ready,

    output frame_beat out,
    input wire out_ready,

    output wire [NODE_ADDR_WIDTH-1:0] bram_node_addr[STAGE_NUM:1],
    input bitmap_node bram_node_data[STAGE_NUM:1],
    output wire [NODE_ADDR_WIDTH-1:0] bram_entry_addr,
    input wire [ENTRY_WIDTH-1:0] bram_entry_data,
    output logic [ENTRY_WIDTH-1:0] next_hop_addr,
    input next_hop_t next_hop_data
);

  frame_beat s[STAGE_NUM:0];
  frame_beat s_reg[STAGE_NUM:0];
  frame_beat s1;
  wire s1_ready;
  wire s_ready[STAGE_NUM:0];
  // bram
  logic node_valid_comb[STAGE_NUM:0];
  logic nexthop_valid[STAGE_NUM:0];
  // pipeline 
  logic [NODE_ADDR_WIDTH-1:0] entry_addr[STAGE_NUM:0];
  logic [NODE_ADDR_WIDTH-1:0] node_addr[STAGE_NUM:0];
  logic node_valid[STAGE_NUM:0];
  logic [127:0] addr_reg[STAGE_NUM:0];
  bitmap_node bram_data_reg[STAGE_NUM:1];

  // pop count
  logic [NODE_ADDR_WIDTH-1:0] pop_count[STAGE_NUM:1];
  logic [NODE_ADDR_WIDTH-1:0] pop_count_entry0[STAGE_NUM:1];
  logic [NODE_ADDR_WIDTH-1:0] pop_count_entry1[STAGE_NUM:1];
  logic [NODE_ADDR_WIDTH-1:0] pop_count_entry2[STAGE_NUM:1];
  logic [NODE_ADDR_WIDTH-1:0] pop_count_entry3[STAGE_NUM:1];
  logic [NODE_ADDR_WIDTH-1:0] pop_count_entry4[STAGE_NUM:1];
  logic [NODE_ADDR_WIDTH-1:0] pop_count_node[STAGE_NUM:1];

  logic [NODE_ADDR_WIDTH-1:0] bram_addr_comb[STAGE_NUM:1];

  typedef enum logic [1:0] { ST_SEND_RECV, ST_CALC, ST_RAM, ST_RAM2 } state_t;

  state_t state[STAGE_NUM:0];
  logic [1:0] s1_state;

  assign s[0] = in;
  assign in_ready = s_ready[0];
  always_comb begin
    out = s1;
    out.valid = s1.valid && s1_state == 0;
  end
  assign s1_ready = out_ready;
  assign s_ready[STAGE_NUM] = (s1_ready && s1_state == 0) || !s[STAGE_NUM].valid;

  assign addr_reg[0] = in.data.ip6.dst;
  assign entry_addr[0] = 0;
  assign nexthop_valid[0] = 0;
  assign node_addr[0] = 0;
  assign node_valid[0] = 1'b1;
  logic [2:0] count_reg[STAGE_NUM:1];
  logic [3:0] now_data_reg[STAGE_NUM:1];

  genvar i;
  generate
    for (i = 1; i <= STAGE_NUM; i = i + 1) begin
      assign s_ready[i-1] = !s[i-1].valid || (s_ready[i] && state[i] == ST_SEND_RECV);

      assign bram_node_addr[i] = state[i] == ST_SEND_RECV ? (s[i-1].valid ? node_addr[i-1] : 0) : node_addr[i];

      always_comb begin
        s[i] = s_reg[i];
        s[i].valid = s_reg[i].valid && state[i] == ST_SEND_RECV;
      end

      always_comb begin
        pop_count[i] = 0;
        pop_count_entry1[i] = 0;
        pop_count_entry2[i] = 0;
        pop_count_entry3[i] = 0;
        pop_count_entry4[i] = 0;
        pop_count_node[i] = 0;
        for (int j = 0; j < 4'b1111; j++) begin
          if (j == 4'b1110) begin
            pop_count_entry0[i] = pop_count[i];
          end
          if (j == 4'd12 + now_data_reg[i][3:3]) begin
            pop_count_entry1[i] = pop_count[i];
          end
          if (j == 4'd8 + now_data_reg[i][3:2]) begin
            pop_count_entry2[i] = pop_count[i];
          end
          if (j == now_data_reg[i][3:1]) begin
            pop_count_entry3[i] = pop_count[i];
          end
          pop_count[i] = pop_count[i] + bram_data_reg[i].entry_mask[j];
        end
        for (int j = 0; j < now_data_reg[i]; ++j) begin
          pop_count_node[i] = pop_count_node[i] + bram_data_reg[i].child_mask[j];
        end
        pop_count_entry4[i] = pop_count_node[i];

        bram_addr_comb[i] = 0;
        node_valid_comb[i] = 0;
        if (!bram_data_reg[i].is_leaf && bram_data_reg[i].child_mask[now_data_reg[i]]) begin
          bram_addr_comb[i] = bram_data_reg[i].child_addr + pop_count_node[i];
          node_valid_comb[i] = 1'b1;
        end
      end

      always_ff @(posedge clk or posedge rst) begin
        if (rst) begin
          s_reg[i] <= 0;
          entry_addr[i] <= 0;
          node_addr[i] <= 0;
          bram_data_reg[i] <= 0;
          addr_reg[i] <= 0;
          count_reg[i] <= 0;
          now_data_reg[i] <= 0;
          node_valid[i] <= 0;
          nexthop_valid[i] <= 0;
          state[i] <= ST_SEND_RECV;
        end else begin
          case (state[i])
            ST_SEND_RECV: begin
              if (s_ready[i]) begin
                s_reg[i] <= s[i-1];
                entry_addr[i] <= entry_addr[i-1];
                bram_data_reg[i] <= bram_node_data[i];
                addr_reg[i] <= addr_reg[i-1];
                node_addr[i] <= node_addr[i-1];
                node_valid[i] <= node_valid[i-1];
                nexthop_valid[i] <= nexthop_valid[i-1];
                if (`should_handle(s[i-1]) && node_valid[i-1]) begin
                  state[i] <= ST_RAM;
                  count_reg[i] <= 0;
                end
              end
            end
            ST_CALC: begin

              // if (!bram_data_reg[i].valid) begin
              //   node_addr[i] <= 0;
              //   node_valid[i] <= 0;
              // end else begin
                
              if (bram_data_reg[i].entry_mask[14]) begin // *
                entry_addr[i] <= bram_data_reg[i].entry_addr + pop_count_entry0[i];
                nexthop_valid[i] <= 1'b1;
              end
              if (bram_data_reg[i].entry_mask[4'd12 + now_data_reg[i][3:3]]) begin // ?*
                entry_addr[i] <= bram_data_reg[i].entry_addr + pop_count_entry1[i];
                nexthop_valid[i] <= 1'b1;
              end
              if (bram_data_reg[i].entry_mask[4'd8 + now_data_reg[i][3:2]]) begin // ??*
                entry_addr[i] <= bram_data_reg[i].entry_addr + pop_count_entry2[i];
                nexthop_valid[i] <= 1'b1;
              end
              if (bram_data_reg[i].entry_mask[now_data_reg[i][3:1]]) begin // ???*
                entry_addr[i] <= bram_data_reg[i].entry_addr + pop_count_entry3[i];
                nexthop_valid[i] <= 1'b1;
              end
              if (bram_data_reg[i].is_leaf && bram_data_reg[i].child_mask[now_data_reg[i]]) begin
                entry_addr[i] <= bram_data_reg[i].child_addr + pop_count_entry4[i];
                nexthop_valid[i] <= 1'b1;
              end
              
              // end

              node_valid[i] <= node_valid_comb[i];
              node_addr[i] <= bram_addr_comb[i];

              count_reg[i] <= count_reg[i] + 1;
              if (count_reg[i] == 2'b11 || !node_valid_comb[i]) begin
                state[i] <= ST_SEND_RECV;
                addr_reg[i] <= addr_reg[i] >> 16;
              end else begin
                state[i] <= ST_RAM2;
              end
            end
            ST_RAM: begin
              bram_data_reg[i] <= bram_node_data[i];
              state[i] <= ST_CALC;
              now_data_reg[i] <= 0;
              case (count_reg[i])
                2'b00: now_data_reg[i] <= addr_reg[i][7:4];
                2'b01: now_data_reg[i] <= addr_reg[i][3:0];
                2'b10: now_data_reg[i] <= addr_reg[i][15:12];
                2'b11: now_data_reg[i] <= addr_reg[i][11:8];
                default: now_data_reg[i] <= 0;
              endcase
            end
            ST_RAM2: begin
              state[i] <= ST_RAM;
            end
            default: state[i] <= ST_SEND_RECV;
          endcase

        end
      end
    end
  endgenerate

  logic [NODE_ADDR_WIDTH-1:0] s1_entry_addr;
  next_hop_t next_hop_data_reg;

  assign bram_entry_addr = s1_state == 0 ? entry_addr[STAGE_NUM] : s1_entry_addr;

  always_ff @(posedge clk or posedge rst) begin
    if (rst) begin
      s1 <= 0;
      s1_entry_addr <= 0;
      s1_state <= 0;
      next_hop_addr <= 0;
      next_hop_data_reg <= 0;
    end else begin
      case (s1_state)
        0: begin
          if (s1_ready) begin
            s1 <= s[STAGE_NUM];
            if (s[STAGE_NUM].meta.next_hop_from_src) begin
              s1.meta.next_hop <= s[STAGE_NUM].data.ip6.src;
            end else if (s[STAGE_NUM].meta.next_hop_from_dst) begin
              s1.meta.next_hop <= s[STAGE_NUM].data.ip6.dst;
            end else if (`should_handle(s[STAGE_NUM])) begin
              if (nexthop_valid[STAGE_NUM]) begin
                s1_state <= 1;
                s1_entry_addr <= entry_addr[STAGE_NUM];
              end else s1.meta.drop <= 1;
            end
          end
        end
        1: begin
          s1_state <= 2;
          next_hop_addr <= bram_entry_data;
        end
        2: begin
          s1_state <= 3;
          next_hop_data_reg <= next_hop_data;
        end
        3: begin
          s1_state <= 0;
          s1.meta.dest <= {1'b0, next_hop_data_reg.dest};
          s1.meta.next_hop <= next_hop_data_reg.direct ? s1.data.ip6.dst : next_hop_data_reg.next_hop;
        end
        default: s1_state <= 0;
      endcase
    end
  end

endmodule
