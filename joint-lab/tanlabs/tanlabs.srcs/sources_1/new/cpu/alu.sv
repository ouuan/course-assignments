`timescale 1ns / 1ps

`include "rv32i_cpu.vh"

module alu (
    input wire [31:0] a,
    input wire [31:0] b,
    input wire opcode_t op,
    output wire [31:0] y

    // zero, sign, overflow flag
    // only valid when op is ADD | SUB
    // output cc_t new_cc
);

    logic [31:0] out;
    // logic of_reg, cf_reg;
    always_comb begin
        // of_reg = 0;
        case (op)
            ADD: begin
                out = a + b;
                // of_reg = (a[31] == b[31]) & (a[31] != out[31]);
                // cf_reg = $unsigned(out) < $unsigned(a);
            end
            SUB: begin
                out = a - b;
                // of_reg = (~a[31] == b[31]) & (b[31] == out[31]);
                // cf_reg = $unsigned(a) < $unsigned(b);
            end
            AND: out = a & b;
            OR : out = a | b;
            XOR: out = a ^ b;
            SLL: out = a << (b & 31);
            SRL: out = a >> (b & 31);
            SRA: out = $signed(a) >>> (b & 31);
            SLT: out = ($signed(a) < $signed(b)) ? 1 : 0;
            SLTU: out = ($unsigned(a) < $unsigned(b)) ? 1 : 0;
            EQU: out = (a == b) ? 1 : 0;
            BSET: out = a | (1 << (b & 31));
            BEXT: out = (a >> (b & 31)) & 1;
            CTZ: begin
                out = 0;
                for (integer i = 0; i < 32; i++) begin
                    if (a[i] & 1) break;
                    out = out + 1;
                end
            end
            CPOP: begin
                out = 0;
                for (integer i = 0; i < 32; i++) begin
                    if (a[i] & 1) out = out + 1;
                end
            end
            default: out = 0;
        endcase
    end

    assign y = out;
    // assign new_cc.cf = cf_reg;
    // assign new_cc.zf = (out == 0);
    // assign new_cc.sf = out[31];
    // assign new_cc.of = of_reg;

endmodule
