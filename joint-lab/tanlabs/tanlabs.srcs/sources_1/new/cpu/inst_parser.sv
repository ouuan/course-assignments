`timescale 1ns / 1ps

`include "rv32i_cpu.vh"

module inst_parser (
    input wire [31:0] in,

    output branch_t branch_type,
    output memory_t mem_width,
    output opcode_t alu_op,
    output wire [4:0] rs1,
    output wire [4:0] rs2,
    output wire [4:0] rd,
    output wire branch_ins,
    output wire jump_ins,
    output wire reg_we,
    output wire mem_ins,
    output wire mem_we,
    output wire [31:0] imm,
    output icode_t icode
);


    icode_t icode_comb;
    opcode_t opcode_comb;
    memory_t memory_comb;
    branch_t branch_comb;
    logic [4:0] rs1_comb, rs2_comb, rd_comb;
    logic [31:0] imm_comb;

    assign branch_type = branch_comb;
    assign mem_width = memory_comb;
    assign alu_op = opcode_comb;
    assign rs1 = rs1_comb;
    assign rs2 = rs2_comb;
    assign rd  = rd_comb;
    assign branch_ins = icode_comb == BRANCH;
    assign jump_ins = (icode_comb == JAL) || (icode_comb == JALR);
    assign reg_we = (icode_comb == CALC) || (icode_comb == CALC_IMM) || (icode_comb == LUI)
                    || (icode_comb == AUIPC) || (icode_comb == LOAD) || (icode_comb == JAL) || (icode_comb == JALR);
    assign mem_ins = (icode_comb == LOAD) || (icode_comb == STORE);
    assign mem_we  = icode_comb == STORE;
    assign imm = imm_comb;
    assign icode = icode_comb;

    always_comb begin

        icode_comb = I_INVALID;
        opcode_comb = OP_INVALID;
        memory_comb = MEM_INVALID;
        branch_comb = TAKEN_IF_ALU_0;
        rs1_comb = 0;
        rs2_comb = 0;
        rd_comb = 0;
        imm_comb = 0;

        // parse instruction class
        case (in[6:0])
            7'b0110011: icode_comb = CALC;
            7'b0010011: icode_comb = CALC_IMM;
            7'b0000011: icode_comb = LOAD;
            7'b0110111: icode_comb = LUI;
            7'b0100011: icode_comb = STORE;
            7'b1100011: icode_comb = BRANCH;
            7'b0010111: icode_comb = AUIPC;
            7'b1101111: icode_comb = JAL;
            7'b1100111: icode_comb = JALR;
            7'b0001111: icode_comb = FENCE;
            // TODO: I_INVALID will run same as INOP, but need to raise error
            default: icode_comb = I_INVALID;
        endcase

        // parse operation code
        case (icode_comb)
            CALC: begin
                // funct7 + funct3
                case ({in[31:25] ,in[14:12]})
                    10'b0000000000: opcode_comb = ADD;
                    10'b0100000000: opcode_comb = SUB;
                    10'b0000000111: opcode_comb = AND;
                    10'b0000000110: opcode_comb = OR;
                    10'b0000000100: opcode_comb = XOR;
                    10'b0000000001: opcode_comb = SLL;
                    10'b0000000101: opcode_comb = SRL;
                    10'b0100000101: opcode_comb = SRA;
                    10'b0000000010: opcode_comb = SLT;
                    10'b0000000011: opcode_comb = SLTU;
                    10'b0010100001: opcode_comb = BSET;
                    10'b0100100101: opcode_comb = BEXT;
                    // TODO: need to raise error
                    default: opcode_comb = OP_INVALID;
                endcase
            end
            CALC_IMM: begin
                case (in[14:12])
                    3'b000: opcode_comb = ADD;
                    // note that there is no SUB operation according to document
                    3'b111: opcode_comb = AND;
                    3'b110: opcode_comb = OR;
                    3'b100: opcode_comb = XOR;
                    3'b001: begin
                        case (in[31:25])
                            7'b0000000: opcode_comb = SLL;
                            7'b0110000: begin
                                case (in[24:20])
                                    5'b00001: opcode_comb = CTZ;
                                    5'b00010: opcode_comb = CPOP;
                                    default: begin
                                        opcode_comb = OP_INVALID;
                                    end
                                endcase
                            end
                            default: begin
                                opcode_comb = OP_INVALID;
                            end
                        endcase
                    end
                    3'b101: begin
                        case (in[31:25])
                            7'b0000000: opcode_comb = SRL;
                            7'b0100000: opcode_comb = SRA;
                            default: begin
                                opcode_comb = OP_INVALID;
                            end
                        endcase
                    end
                    3'b010: opcode_comb = SLT;
                    3'b011: opcode_comb = SLTU;
                    // TODO: need to raise error
                    default: opcode_comb = OP_INVALID;
                endcase

            end
            LOAD: begin
                opcode_comb = ADD;
                case (in[14:12])
                    3'd0: memory_comb = BYTE;
                    3'd1: memory_comb = HALF;
                    3'd2: memory_comb = WORD;
                    3'd4: memory_comb = BYTE_UNSIGNED;
                    3'd5: memory_comb = HALF_UNSIGNED;
                    // TODO: need to raise error
                    default: memory_comb = MEM_INVALID;
                endcase
            end
            STORE: begin
                opcode_comb = ADD;
                case (in[14:12])
                    3'd0: memory_comb = BYTE;
                    3'd1: memory_comb = HALF;
                    3'd2: memory_comb = WORD;
                    // TODO: need to raise error
                    default: memory_comb = MEM_INVALID;
                endcase
            end
            BRANCH: begin
                case (in[14:12])
                    3'd0: begin
                        branch_comb = TAKEN_IF_ALU_1;
                        opcode_comb = EQU;
                    end
                    3'd1: begin
                        branch_comb = TAKEN_IF_ALU_0;
                        opcode_comb = EQU;
                    end 
                    3'd4: begin
                        branch_comb = TAKEN_IF_ALU_1;
                        opcode_comb = SLT;
                    end
                    3'd5: begin
                        branch_comb = TAKEN_IF_ALU_0;
                        opcode_comb = SLT;
                    end
                    3'd6: begin
                        branch_comb = TAKEN_IF_ALU_1;
                        opcode_comb = SLTU;
                    end
                    3'd7: begin
                        branch_comb = TAKEN_IF_ALU_0;
                        opcode_comb = SLTU;
                    end
                    // TODO: need to raise error
                    default: branch_comb = TAKEN_IF_ALU_0;
                endcase
            end
            AUIPC, LUI, JAL, JALR: begin
                opcode_comb = ADD;
            end
            FENCE: begin
                opcode_comb = ADD;
            end
            default: opcode_comb = OP_INVALID;
        endcase

        // parse register rs1
        case (icode_comb)
            CALC, CALC_IMM, LOAD, STORE, BRANCH, JALR: rs1_comb = in[19:15];
            default: rs1_comb = 0;
        endcase

        // parse register rs2
        case (icode_comb)
            CALC, STORE, BRANCH: rs2_comb = in[24:20];
            default: rs2_comb = 0;
        endcase

        // parse register rd
        case (icode_comb)
            CALC, CALC_IMM, JAL, JALR, LUI, AUIPC: begin
                rd_comb = in[11:7];
            end
            LOAD: begin
                rd_comb = in[11:7];
            end
            default: begin
                rd_comb = 0;
            end
        endcase

        // parse imm
        case (icode_comb)
            CALC_IMM, LOAD, JALR: imm_comb = {{(20){in[31]}}, in[31:20]};
            STORE: imm_comb = {{(20){in[31]}}, in[31:25], in[11:7]};
            BRANCH: imm_comb = {{(19){in[31]}}, in[31], in[7], in[30:25], in[11:8], 1'b0};
            JAL: imm_comb = {{(12){in[31]}}, in[31], in[19:12], in[20], in[30:21], 1'b0};
            LUI, AUIPC: imm_comb = {in[31:12], 12'b0};
            default: imm_comb = 0;
        endcase
    end


endmodule
