`timescale 1ns / 1ps

`include "rv32i_cpu.vh"

module regfile (
    input wire clk,
    input wire rst,

    input wire [4:0] raddr_a,
    output wire [31:0] rdata_a,
    input wire [4:0] raddr_b,
    output wire [31:0] rdata_b,
    // input wire [4:0] raddr_c,
    // output wire [31:0] rdata_c,
    // input wire [4:0] raddr_f,
    // output wire [31:0] rdata_f,

    input wire pip_state w
);
    reg [31:0] regs[32];
    integer i;

    always_ff @(posedge clk or posedge rst) begin
        if (rst) begin
            for (i = 0; i < 32; i = i + 1) begin
                regs[i] <= 0;
            end
        end
        else begin
            if (w.reg_we && w.rd != 0) begin
                regs[w.rd] <= w.val_w;
            end
        end
    end
    assign rdata_a =
        (raddr_a == 0) ? 0 :
        (raddr_a == w.rd) ? w.val_w :
        regs[raddr_a];

    assign rdata_b =
        (raddr_b == 0) ? 0 :
        (raddr_b == w.rd) ? w.val_w :
        regs[raddr_b];

endmodule
