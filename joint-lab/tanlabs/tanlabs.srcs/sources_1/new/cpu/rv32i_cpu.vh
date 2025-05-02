`ifndef _ENUM_HDR_VH_
`define _ENUM_HDR_VH_

localparam SYS_CLOCK_FREQ = 125_000_000;

localparam SRAM_BASE = 32'h80000000;
localparam ROUTER2CPU_BASE = 32'h80600000;
localparam ROUTER2CPU_PACKET_SIZE = 1 << 11;
localparam ROUTER2CPU_PACKET_COUNT = 1 << 9;

typedef enum logic [3:0] {
    ADD,
    SUB,
    AND,
    OR ,
    XOR,
    SLL,
    SRL,
    SRA,
    SLT,
    SLTU,
    EQU,
    BSET,
    BEXT,
    CPOP,
    CTZ,
    OP_INVALID
} opcode_t;

typedef enum logic {
    TAKEN_IF_ALU_0,
    TAKEN_IF_ALU_1
} branch_t;

typedef enum logic [2:0] {
    BYTE,
    HALF,
    WORD,
    BYTE_UNSIGNED,
    HALF_UNSIGNED,
    MEM_INVALID
} memory_t;


typedef enum logic [6:0] {
    // add, sub, xor, or, and, sll, srl, sra, slt, sltu
    CALC = 7'b0110011,

    // addi, subi, xori, ori, andi, slli, srli, srai, slti, sltiu 
    CALC_IMM = 7'b0010011,

    // lb, lh, lw, lbu, lhu
    LOAD = 7'b0000011,

    // lui
    LUI = 7'b0110111,

    // sb, sh, sw
    STORE = 7'b0100011,

    // beq, bne, blt, bge, bltu, bgeu
    BRANCH = 7'b1100011,

    // auipc
    AUIPC = 7'b0010111,

    // jal
    JAL = 7'b1101111,

    // jalr
    JALR = 7'b1100111,

    FENCE = 7'b0001111,

    INOP,

    I_INVALID
} icode_t;

typedef enum logic [7:0] {
    ERR_NONE,
    ERR_FAILED
} error_t;

typedef struct packed {
    logic [31:0] inst;
    branch_t branch_type; // branch type
    memory_t mem_width; // mem width
    opcode_t alu_op; // alu op
    logic [31:0] pc; // pc
    logic [4:0] rs1; // rs1
    logic [4:0] rs2; // rs2
    logic [31:0] val_a; // alu input a
    logic [31:0] val_b; // alu input b
    logic [4:0] rd; // rd
    logic branch_ins; // is branch ins
    logic jump_ins; // is jump ins
    logic reg_we; // is reg write ins
    logic [31:0] val_w; // write data
    logic mem_ins; // is mem ins
    logic mem_we; // is mem write ins
    logic [31:0] mem_addr; // mem addr
    logic [31:0] imm; // imm of ins
    logic set_pc; // is pc changed
    logic [31:0] untaken_pc; // pc = untaken_pc;

    logic stall;
    logic bubble;

    logic rst;

    error_t err;
} pip_state;

typedef struct packed {
    logic cf;
    logic zf;
    logic sf;
    logic of;
} cc_t;

// master signal input
typedef struct packed {
    logic ack;
    logic [31:0] dat;
} wbm_signal_i;

// master signal output
typedef struct packed {
    logic cyc;
    logic stb;
    logic [31:0] adr;
    logic [31:0] dat;
    logic [3:0] sel;
    logic we;
} wbm_signal_o;

`endif
