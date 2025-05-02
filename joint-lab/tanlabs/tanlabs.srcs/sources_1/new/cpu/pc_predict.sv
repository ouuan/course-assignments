`timescale 1ns / 1ps

`include "rv32i_cpu.vh"

module pc_predict (
    input wire clk,
    input wire rst,
    input wire stall,
    input wire [31:0] inst,
    input wire [31:0] pc,
    input wire [31:0] init_pc,
    input wire wrong_pc_pred,
    input wire branch_ins,
    output wire set_pc,
    output wire [31:0] pred_pc,
    output wire [31:0] untaken_pc
);

    logic set_pc_comb;
    logic [31:0] pred_pc_comb; 
    logic [31:0] untaken_pc_comb;
    logic [31:0] imm_comb;

    assign set_pc = set_pc_comb;
    assign pred_pc = pred_pc_comb;
    assign untaken_pc = untaken_pc_comb;

    typedef enum logic [1:0] { SNT, WNT, WT, ST } predict_t;
    predict_t p_state;

    always_ff @(posedge clk) begin
        if (rst) begin
            p_state <= ST;
        end else if (!stall && branch_ins) begin
            if (wrong_pc_pred) begin
                case (p_state)
                    SNT: p_state <= WNT; // pred NT, infact T
                    WNT: p_state <= ST; // pred NT, infact T
                    WT: p_state <= SNT; // pred T, infact NT
                    ST: p_state <= WT;  // pred T, infact NT
                    default: p_state <= ST;
                endcase
            end else begin
                case (p_state)
                    SNT: p_state <= SNT; // pred NT, infact NT
                    WNT: p_state <= SNT; // pred NT, infact NT
                    WT: p_state <= ST;   // pred T, infact T
                    ST: p_state <= ST;   // pred T, infact T
                    default: p_state <= ST;
                endcase
            end
        end
    end

    always_comb begin
        set_pc_comb = 1'b0;
        pred_pc_comb = pc + 4;
        untaken_pc_comb = 0;
        if (inst == '0) begin
            pred_pc_comb = init_pc;
        end else if (inst[6:0] == 7'b1100011) begin // branch inst
            imm_comb = {{(19){inst[31]}}, inst[31], inst[7], inst[30:25], inst[11:8], 1'b0};
            if (p_state == SNT || p_state == WNT) begin
                untaken_pc_comb = pc + imm_comb;
                pred_pc_comb = pc + 4;
                set_pc_comb = 1'b0;
            end else begin
                pred_pc_comb = pc + imm_comb;
                untaken_pc_comb = pc + 4;
                set_pc_comb = 1'b1;
            end
        end else if (inst[6:0] == 7'b1101111) begin // jal inst
            imm_comb = {{(12){inst[31]}}, inst[31], inst[19:12], inst[20], inst[30:21], 1'b0};
            pred_pc_comb = pc + imm_comb;
            set_pc_comb = 1'b1;
        end else if (inst[6:0] == 7'b1100111) begin // jalr inst
            imm_comb = {{(20){inst[31]}}, inst[31:20]};
        end
    end

endmodule
