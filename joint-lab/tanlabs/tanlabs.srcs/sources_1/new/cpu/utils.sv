`timescale 1ns / 1ps

`include "rv32i_cpu.vh"

module reg_ctrl #(
    parameter DATA_WIDTH = 32
) (
    output reg [DATA_WIDTH-1:0] out,
    input wire [DATA_WIDTH-1:0] in,
    input wire we,
    input wire rst,
    input wire [DATA_WIDTH-1:0] rst_val,
    input wire clk
);
    always_ff @(posedge clk) begin
        if (rst) begin
            out <= rst_val;
        end else if (we) begin
            out <= in;
        end
    end
endmodule

module reg_pipe #(
    parameter DATA_WIDTH = 32
) (
    output reg [DATA_WIDTH-1:0] out,
    input wire [DATA_WIDTH-1:0] in,
    input wire stall,
    input wire bubble,
    input wire [DATA_WIDTH-1:0] bubble_val,
    input wire clk
);
    reg_ctrl #(DATA_WIDTH) r(
        .out        (out),
        .in         (in),
        .we         (~stall),
        .rst        (bubble),
        .rst_val    (bubble_val),
        .clk        (clk)
    );
endmodule

module reg_icode (
    output icode_t out,
    input wire icode_t in,
    input wire stall,
    input wire bubble,
    input wire icode_t bubble_val,
    input wire clk
);
    always_ff @(posedge clk) begin
        if (bubble) begin
            out <= bubble_val;
        end else if (~stall) begin
            out <= in;
        end
    end
endmodule

module reg_memory (
    output memory_t out,
    input wire memory_t in,
    input wire stall,
    input wire bubble,
    input wire memory_t bubble_val,
    input wire clk
);
    always_ff @(posedge clk) begin
        if (bubble) begin
            out <= bubble_val;
        end else if (~stall) begin
            out <= in;
        end
    end
endmodule

module reg_branch (
    output branch_t out,
    input wire branch_t in,
    input wire stall,
    input wire bubble,
    input wire branch_t bubble_val,
    input wire clk
);
    always_ff @(posedge clk) begin
        if (bubble) begin
            out <= bubble_val;
        end else if (~stall) begin
            out <= in;
        end
    end
endmodule

module reg_opcode (
    output opcode_t out,
    input wire opcode_t in,
    input wire stall,
    input wire bubble,
    input wire opcode_t bubble_val,
    input wire clk
);
    always_ff @(posedge clk) begin
        if (bubble) begin
            out <= bubble_val;
        end else if (~stall) begin
            out <= in;
        end
    end
endmodule

module reg_err (
    output error_t out,
    input wire error_t in,
    input wire stall,
    input wire bubble,
    input wire error_t bubble_val,
    input wire clk
);
    always_ff @(posedge clk) begin
        if (bubble) begin
            out <= bubble_val;
        end else if (~stall) begin
            out <= in;
        end
    end
endmodule


