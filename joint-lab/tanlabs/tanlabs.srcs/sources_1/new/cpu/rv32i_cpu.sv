`timescale 1ns / 1ps

`include "rv32i_cpu.vh"

module rv32i_cpu (
    input wire clk,
    input wire rst,

    input wire inst_we,
    input wire [12:0] inst_addr,
    input wire [31:0] inst_wdata,

    // wishbone master
    input wire wbm_signal_i m_wb_i,
    output wire wbm_signal_o m_wb_o
);
    // we use lowercase letter to represent the output of each tier of pipeline
    // while the uppercase letter is input
    pip_state F, D, E, M, W;
    pip_state f, d, e, m, w;

    typedef enum {
        F_INIT,
        F_WAIT
    } f_state_t;

    f_state_t f_state;

    typedef enum {
        M_INIT,
        M_WAIT
    } m_state_t;

    m_state_t m_state;

    logic glb_stall;
    assign glb_stall =
        (f_state == F_WAIT) |
        (m_state == M_WAIT);

    assign F.rst = (F.bubble & ~glb_stall) | rst;
    assign D.rst = (D.bubble & ~glb_stall) | rst;
    assign E.rst = (E.bubble & ~glb_stall) | rst;
    assign M.rst = (M.bubble & ~glb_stall) | rst;
    assign W.rst = (W.bubble & ~glb_stall) | rst;


    // waiting for memory
    logic wait_val_m;

    logic wrong_pc_pred;
    assign wrong_pc_pred = M.set_pc;

    // Stall, Bubble, Flush controll
    // Stall means do nothing in this circle
    // Bubble means next circle this stage set to NOP
    assign F.bubble =
        0;
    assign F.stall =
        D.stall;

    assign D.bubble =
        wrong_pc_pred;
    assign D.stall =
        E.stall |
        wait_val_m;

    assign E.bubble =
        wait_val_m |
        wrong_pc_pred;
    assign E.stall =
        M.stall;

    assign M.bubble =
        wrong_pc_pred;
    assign M.stall =
        W.stall;

    assign W.bubble =
        0;
    assign W.stall =
        glb_stall |
        0;

    // Fetch stage
    logic [31:0] pred_pc;

    // select pc
    assign f.pc =
        rst ? 32'h80000000 :
        M.set_pc ? M.untaken_pc :
        pred_pc;

    logic [31:0] inst_pc;
    pc_predict pc_predict_i (
        .clk(clk),
        .rst(rst),
        .wrong_pc_pred(wrong_pc_pred),
        .branch_ins(M.branch_ins),
        .stall(M.stall),
        .inst(f.inst),
        .pc(inst_pc),
        .init_pc(32'h80000000),
        .pred_pc(pred_pc),
        .untaken_pc(f.untaken_pc),
        .set_pc(f.set_pc)
    );

    bram_inst bram_inst_i (
        .clka(clk),
        .ena(inst_we),
        .wea(inst_we),
        .addra(inst_addr),
        .dina(inst_wdata),
        .clkb(clk),
        .addrb(F.stall ? inst_pc[14:2] : f.pc[14:2]),
        .doutb(f.inst)
    );

    always_ff @(posedge clk, posedge rst) begin
        if (rst) begin
            f_state <= F_INIT;
            inst_pc <= 32'h80000000;
        end else begin
            case (f_state)
                F_INIT: begin
                    if (!F.stall) begin
                        inst_pc <= f.pc;
                    end
                end
                default: f_state <= F_INIT;
            endcase
        end
    end

    reg_pipe #(32) D_pc_reg(D.pc, inst_pc, D.stall, D.rst, '0, clk);
    reg_pipe #(32) D_inst_reg(D.inst, f.inst, D.stall, D.rst, '0, clk);
    reg_pipe #(32) D_untaken_pc_reg(D.untaken_pc, f.untaken_pc, D.stall, D.rst, '0, clk);
    reg_pipe #(1)  D_set_pc_reg(D.set_pc, f.set_pc, D.stall, D.rst, '0, clk);

    icode_t D_icode;

    inst_parser inst_parser_i (
        .in (D.inst),
        .branch_type(d.branch_type),
        .mem_width(d.mem_width),
        .alu_op(d.alu_op),
        .rs1(d.rs1),
        .rs2(d.rs2),
        .rd(d.rd),
        .branch_ins(d.branch_ins),
        .jump_ins(d.jump_ins),
        .reg_we(d.reg_we),
        .mem_ins(d.mem_ins),
        .mem_we(d.mem_we),
        .imm(d.imm),
        .icode(D_icode)
    );

    logic [31:0] rf_rdata_a;
    logic [31:0] rf_rdata_b;

    regfile regfile_i (
        .clk(clk),
        .rst(rst),
        .raddr_a(d.rs1),
        .raddr_b(d.rs2),
        .rdata_a(rf_rdata_a),
        .rdata_b(rf_rdata_b),
        .w(w)
    );

    // forward
    // NOTE: if d.rs1 == 0 means read reg0 or do not read regs in parser
    // wrong_pc_pred mean ins in D is invalid
    assign wait_val_m =
        (!(d.rs1 == 0) & (d.rs1 == E.rd) & E.mem_ins & !wrong_pc_pred) |
        (!(d.rs2 == 0) & (d.rs2 == E.rd) & E.mem_ins & !wrong_pc_pred);

    assign d.val_a =
        (D_icode == AUIPC || D_icode == JAL) ? D.pc :
        (d.rs1 == 0) ? 0 :
        (d.rs1 == E.rd && !E.mem_ins) ? e.val_w : // rs1 == E.rd, E is not LOAD, rs1 = aluout
        (d.rs1 == E.rd && E.mem_ins) ? 0 : // wait value, stall till m.val_m
        (d.rs1 == M.rd && !M.mem_ins) ? M.val_w : // rs1 == M.rd, M is not LOAD, rs1 = aluout
        (d.rs1 == M.rd && M.mem_ins) ? m.val_w :  // rs1 == M.rd, M is LOAD, rs1 = memout
        rf_rdata_a;

    assign d.val_b =
        (D_icode == AUIPC || D_icode == LUI || D_icode == LOAD || D_icode == STORE || D_icode == CALC_IMM) ? d.imm :
        (D_icode == JALR || D_icode == JAL) ? d.imm :
        (d.rs2 == 0) ? 0 :
        (d.rs2 == E.rd && !E.mem_ins) ? e.val_w : // rs1 == E.rd, E is not LOAD, rs1 = aluout
        (d.rs2 == E.rd && E.mem_ins) ? 0 : // wait value, stall till m.val_m
        (d.rs2 == M.rd && !M.mem_ins) ? M.val_w : // rs1 == M.rd, M is not LOAD, rs1 = aluout
        (d.rs2 == M.rd && M.mem_ins) ? m.val_w :  // rs1 == M.rd, M is LOAD, rs1 = memout
        rf_rdata_b;

    // STORE: write rs2 to mem
    assign d.val_w =
        (D_icode != STORE) ? 0 :
        (d.rs2 == 0) ? 0 :
        (d.rs2 == E.rd && !E.mem_ins) ? e.val_w : // rs1 == E.rd, E is not LOAD, rs1 = aluout
        (d.rs2 == E.rd && E.mem_ins) ? 0 : // wait value, stall till m.val_m
        (d.rs2 == M.rd && !M.mem_ins) ? M.val_w : // rs1 == M.rd, M is not LOAD, rs1 = aluout
        (d.rs2 == M.rd && M.mem_ins) ? m.val_w :  // rs1 == M.rd, M is LOAD, rs1 = memout
        rf_rdata_b;


    // Execute stage
    // reg_icode E_icode_reg(E.icode, D.icode, E.stall, E.rst, INOP, clk);
    reg_branch E_branch_type_reg(E.branch_type, d.branch_type, E.stall, E.rst, TAKEN_IF_ALU_0, clk);
    reg_memory E_mem_width_reg(E.mem_width, d.mem_width, E.stall, E.rst, MEM_INVALID, clk);
    reg_opcode E_alu_op_reg(E.alu_op, d.alu_op, E.stall, E.rst, OP_INVALID, clk);

    reg_pipe #(32) E_pc_reg(E.pc, D.pc, E.stall, E.rst, '0, clk);
    reg_pipe #(32) E_untaken_pc_reg(E.untaken_pc, D.untaken_pc, E.stall, E.rst, '0, clk);
    reg_pipe #(1)  E_set_pc_reg(E.set_pc, D.set_pc, E.stall, E.rst, '0, clk);

    reg_pipe #(5) E_rs1_reg(E.rs1, d.rs1, E.stall, E.rst, '0, clk);
    reg_pipe #(5) E_rs2_reg(E.rs2, d.rs2, E.stall, E.rst, '0, clk);
    reg_pipe #(32) E_imm_reg(E.imm, d.imm, E.stall, E.rst, '0, clk);
    reg_pipe #(5) E_rd_reg(E.rd, d.rd, E.stall, E.rst, '0, clk);
    reg_pipe #(32) E_val_w_reg(E.val_w, d.val_w, E.stall, E.rst, '0, clk);
    reg_pipe #(32) E_val_a_reg(E.val_a, d.val_a, E.stall, E.rst, '0, clk);
    reg_pipe #(32) E_val_b_reg(E.val_b, d.val_b, E.stall, E.rst, '0, clk);
    reg_pipe #(1) E_branch_ins_reg(E.branch_ins, d.branch_ins, E.stall, E.rst, '0, clk);
    reg_pipe #(1) E_jump_ins_reg(E.jump_ins, d.jump_ins, E.stall, E.rst, '0, clk);
    reg_pipe #(1) E_reg_we_reg(E.reg_we, d.reg_we, E.stall, E.rst, '0, clk);
    reg_pipe #(1) E_mem_ins_reg(E.mem_ins, d.mem_ins, E.stall, E.rst, '0, clk);
    reg_pipe #(1) E_mem_we_reg(E.mem_we, d.mem_we, E.stall, E.rst, '0, clk);
    reg_err E_err_reg(E.err, d.err, E.stall, E.rst, ERR_NONE, clk);

    logic [31:0] alu_out;

    alu alu_i (
        .a(E.val_a),
        .b(E.val_b),
        .op(E.alu_op),
        .y(alu_out)
    );

    // compute if jump pc
    always_comb begin
        // E.set_pc == 1 means this branch / jump inst was predicted jump
        // jalr E.set_pc == 0
        e.set_pc = E.jump_ins ? !E.set_pc : 0;
        // e.set_pc == 1 means jalr
        e.untaken_pc = e.set_pc ? {alu_out[31:1], 1'b0} : E.untaken_pc;
        if (E.branch_ins) begin
            // taken if alu 1, E.set_pc = 1 => taken if alu 0
            case (E.branch_type)
                TAKEN_IF_ALU_1: e.set_pc = E.set_pc ^ alu_out[0];
                TAKEN_IF_ALU_0: e.set_pc = E.set_pc ^ !alu_out[0];
                default: e.set_pc = 0;
            endcase
        end
    end

    always_comb begin
        e.val_w = E.val_w;
        e.mem_addr = 0;
        if (E.jump_ins) begin // jump ins, write pc + 4
            e.val_w = E.pc + 4;
        end else if (E.mem_ins) begin // mem ins, write val_b solved in ID
            e.mem_addr = alu_out;
        end else begin // other ins, write alu out
            e.val_w = alu_out;
        end
    end


    // Memory stage
    reg_memory M_mem_width_reg(M.mem_width, E.mem_width, M.stall, M.rst, MEM_INVALID, clk);
    reg_pipe #(5) M_rd_reg(M.rd, E.rd, M.stall, M.rst, 0, clk);
    reg_pipe #(1) M_reg_we_reg(M.reg_we, E.reg_we, M.stall, M.rst, '0, clk);
    reg_pipe #(1) M_mem_ins_reg(M.mem_ins, E.mem_ins, M.stall, M.rst, '0, clk);
    reg_pipe #(1) M_mem_we_reg(M.mem_we, E.mem_we, M.stall, M.rst, '0, clk);
    reg_pipe #(32) M_pc_reg(M.pc, E.pc, M.stall, M.rst, '0, clk);
    reg_pipe #(1) M_branch_ins_reg(M.branch_ins, E.branch_ins, M.stall, M.rst, '0, clk);
    reg_pipe #(1) M_jump_ins_reg(M.jump_ins, E.jump_ins, M.stall, M.rst, '0, clk);

    reg_pipe #(1) M_set_pc_reg(M.set_pc, e.set_pc, M.stall, M.rst, '0, clk);
    reg_pipe #(32) M_untaken_pc_reg(M.untaken_pc, e.untaken_pc, M.stall, M.rst, '0, clk);
    reg_pipe #(32) M_val_w_reg(M.val_w, e.val_w, M.stall, M.rst, '0, clk);
    reg_pipe #(32) M_mem_addr_reg(M.mem_addr, e.mem_addr, M.stall, M.rst, '0, clk);

    reg_err M_err_reg(M.err, e.err, M.stall, M.rst, ERR_NONE, clk);

    wbm_signal_o m_wb_o_reg;
    logic m_wb_o_en;
    logic [1:0] m_offset;
    assign m_wb_o_reg.cyc = m_wb_o_en;
    assign m_wb_o_reg.stb = m_wb_o_en;

    assign m_wb_o_reg.adr = M.mem_addr;
    assign m_offset = m_wb_o_reg.adr[1:0];
    assign m_wb_o_reg.dat =
        (!M.mem_ins || !M.mem_we) ? 0 :
        (M.mem_width == BYTE) ? (
            (m_offset == 2'd0) ? {24'b0, M.val_w[ 7:0]       } :
            (m_offset == 2'd1) ? {16'b0, M.val_w[ 7:0],  8'b0} :
            (m_offset == 2'd2) ? { 8'b0, M.val_w[ 7:0], 16'b0} :
            (m_offset == 2'd3) ? {       M.val_w[ 7:0], 24'b0} : 0
            ) :
        (M.mem_width == HALF) ? (
            (m_offset == 2'd0) ? {16'b0, M.val_w[15:0]       } :
            (m_offset == 2'd1) ? { 8'b0, M.val_w[15:0],  8'b0} :
            (m_offset == 2'd2) ? {       M.val_w[15:0], 16'b0} : 0
            ) :
        (M.mem_width == WORD) ? M.val_w[31:0] : 32'bz;
    assign m_wb_o_reg.sel =
        (M.mem_width == BYTE || M.mem_width == BYTE_UNSIGNED) ? 4'b0001 << m_offset :
        (M.mem_width == HALF || M.mem_width == HALF_UNSIGNED) ? 4'b0011 << m_offset :
        (M.mem_width == WORD) ? 4'b1111 : 32'bz;
    assign m_wb_o_reg.we = M.mem_we;
    assign m_wb_o = m_wb_o_reg;

    logic [31:0] M_val_m;

    always_ff @(posedge clk, posedge rst) begin
        if (rst) begin
            m_wb_o_en <= 0;
            M_val_m <= 0;
            m_state <= M_INIT;
        end else begin
            case (m_state)
                M_INIT: begin
                    if (!M.rst && !M.stall && E.mem_ins) begin
                        m_wb_o_en <= 1;
                        m_state <= M_WAIT;
                    end
                end
                M_WAIT: begin
                    if (m_wb_i.ack) begin
                        m_wb_o_en <= 0;
                        if (!M.mem_we) begin // load
                            case (M.mem_width)
                                BYTE, BYTE_UNSIGNED: begin
                                    case (m_offset)
                                        2'd0: M_val_m <= {M.mem_width == BYTE ? {(24){m_wb_i.dat[ 7]}} : 24'b0, m_wb_i.dat[ 7: 0]};
                                        2'd1: M_val_m <= {M.mem_width == BYTE ? {(24){m_wb_i.dat[15]}} : 24'b0, m_wb_i.dat[15: 8]};
                                        2'd2: M_val_m <= {M.mem_width == BYTE ? {(24){m_wb_i.dat[23]}} : 24'b0, m_wb_i.dat[23:16]};
                                        2'd3: M_val_m <= {M.mem_width == BYTE ? {(24){m_wb_i.dat[31]}} : 24'b0, m_wb_i.dat[31:24]};
                                        default: M_val_m <= 0;
                                    endcase
                                end
                                HALF, HALF_UNSIGNED: begin
                                    case (m_offset)
                                        2'd0: M_val_m <= {M.mem_width == HALF ? {(16){m_wb_i.dat[15]}} : 16'b0, m_wb_i.dat[15: 0]};
                                        2'd1: M_val_m <= {M.mem_width == HALF ? {(16){m_wb_i.dat[23]}} : 16'b0, m_wb_i.dat[23: 8]};
                                        2'd2: M_val_m <= {M.mem_width == HALF ? {(16){m_wb_i.dat[31]}} : 16'b0, m_wb_i.dat[31:16]};
                                        default: begin
                                            // exception
                                        end
                                    endcase
                                end
                                WORD: M_val_m <= m_wb_i.dat;
                                default: begin
                                    // handle error
                                end
                            endcase
                        end
                        m_state <= M_INIT;
                    end
                end
                default: m_state <= M_INIT;
            endcase
        end
    end

    assign m.val_w = (M.mem_ins && !M.mem_we) ? M_val_m : M.val_w;

    // Write stage
    reg_pipe #(5) W_rd_reg(W.rd, M.rd, W.stall, W.rst, '0, clk);
    reg_pipe #(1) W_reg_we_reg(W.reg_we, M.reg_we, W.stall, W.rst, '0, clk);

    reg_pipe #(32) W_val_w_reg(W.val_w, m.val_w, W.stall, W.rst, '0, clk);
    reg_err W_err_reg(W.err, m.err, W.stall, W.rst, ERR_NONE, clk);
    assign w = W;


endmodule
