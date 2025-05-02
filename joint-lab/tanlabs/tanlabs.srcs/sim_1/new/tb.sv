`timescale 1ps / 1ps

module tb
#(
    parameter FAST_BEHAV = 1,  // Fast behavior simulation?
    parameter DATA_WIDTH = 64,
    parameter ID_WIDTH = 3
)
(
);

    logic reset;

    wire clk_125M;

    clock clock_i(
        .clk_125M(clk_125M)
    );

    initial begin
        reset = 1;
        #50
        reset = 0;
    end

    wire [3:0] sfp_tb2dut_p;
    wire [3:0] sfp_tb2dut_n;
    wire [3:0] sfp_dut2tb_p;
    wire [3:0] sfp_dut2tb_n;

    generate
        if (!FAST_BEHAV)
        begin : sfp_model
            wire [DATA_WIDTH - 1:0] in_data;
            wire [DATA_WIDTH / 8 - 1:0] in_keep;
            wire in_last;
            wire [DATA_WIDTH / 8 - 1:0] in_user;
            wire [ID_WIDTH - 1:0] in_id;
            wire in_valid;
            wire in_ready;

            axis_model axis_model_i(
                .clk(clk_125M),
                .reset(reset),

                .m_data(in_data),
                .m_keep(in_keep),
                .m_last(in_last),
                .m_user(in_user),
                .m_id(in_id),
                .m_valid(in_valid),
                .m_ready(in_ready)
            );

            wire [DATA_WIDTH - 1:0] out_data;
            wire [DATA_WIDTH / 8 - 1:0] out_keep;
            wire out_last;
            wire [DATA_WIDTH / 8 - 1:0] out_user;
            wire [ID_WIDTH - 1:0] out_dest;
            wire out_valid;
            wire out_ready;

            sim_axis2sfp sim_axis2sfp_i(
                .reset(reset),
                .clk_125M(clk_125M),

                .s_data(in_data),
                .s_keep(in_keep),
                .s_last(in_last),
                .s_user(in_user),
                .s_dest(in_id),
                .s_valid(in_valid),
                .s_ready(in_ready),

                .m_data(out_data),
                .m_keep(out_keep),
                .m_last(out_last),
                .m_user(out_user),
                .m_id(out_dest),
                .m_valid(out_valid),
                .m_ready(out_ready),

                .sfp_rx_p(sfp_dut2tb_p),
                .sfp_rx_n(sfp_dut2tb_n),
                .sfp_tx_p(sfp_tb2dut_p),
                .sfp_tx_n(sfp_tb2dut_n)
            );

            axis_receiver axis_receiver_i(
                .clk(clk_125M),
                .reset(reset),

                .s_data(out_data),
                .s_keep(out_keep),
                .s_last(out_last),
                .s_user(out_user),
                .s_dest(out_dest),
                .s_valid(out_valid),
                .s_ready(out_ready)
            );
        end
        else
        begin
            assign sfp_tb2dut_p = 0;
            assign sfp_tb2dut_n = 0;
        end
    endgenerate

    wire [31:0] base_ram_data;
    wire [20:0] base_ram_addr;
    wire [3:0] base_ram_be_n;
    wire base_ram_ce_n;
    wire base_ram_oe_n;
    wire base_ram_we_n;

    wire uart_tx;
    wire uart_rx;

    tanlabs
    #(
        .SIM(FAST_BEHAV)
    )
    dut(
        .RST(reset),

        .dip_sw(0),
        .BTN(0),

        .gtclk_125_p(clk_125M),
        .gtclk_125_n(~clk_125M),

        .led(),

        .uart_tx,
        .uart_rx,

        .base_ram_addr,
        .base_ram_data,
        .base_ram_ce_n,
        .base_ram_oe_n,
        .base_ram_we_n,
        .base_ram_be_n,

        .sfp_rx_los(4'd0),
        .sfp_rx_p(sfp_tb2dut_p),
        .sfp_rx_n(sfp_tb2dut_n),
        .sfp_tx_dis(),
        .sfp_tx_p(sfp_dut2tb_p),
        .sfp_tx_n(sfp_dut2tb_n),
        .sfp_link(),
        .sfp_act(),

        .sfp_sda(1'b0),
        .sfp_scl(1'b0)
    );

    uart_model uart (
        .rxd(uart_tx),
        .txd(uart_rx)
    );

    sram_model low0 (
        .DataIO(base_ram_data[15:0]),
        .Address(base_ram_addr[19:0]),
        .OE_n(base_ram_addr[20] || base_ram_oe_n),
        .CE_n(base_ram_addr[20] || base_ram_ce_n),
        .WE_n(base_ram_addr[20] || base_ram_we_n),
        .LB_n(base_ram_be_n[0]),
        .UB_n(base_ram_be_n[1])
    );
    sram_model low1 (
        .DataIO(base_ram_data[31:16]),
        .Address(base_ram_addr[19:0]),
        .OE_n(base_ram_addr[20] || base_ram_oe_n),
        .CE_n(base_ram_addr[20] || base_ram_ce_n),
        .WE_n(base_ram_addr[20] || base_ram_we_n),
        .LB_n(base_ram_be_n[2]),
        .UB_n(base_ram_be_n[3])
    );
    sram_model high0 (
        .DataIO(base_ram_data[15:0]),
        .Address(base_ram_addr[19:0]),
        .OE_n(!base_ram_addr[20] || base_ram_oe_n),
        .CE_n(!base_ram_addr[20] || base_ram_ce_n),
        .WE_n(!base_ram_addr[20] || base_ram_we_n),
        .LB_n(base_ram_be_n[0]),
        .UB_n(base_ram_be_n[1])
    );
    sram_model high1 (
        .DataIO(base_ram_data[31:16]),
        .Address(base_ram_addr[19:0]),
        .OE_n(!base_ram_addr[20] || base_ram_oe_n),
        .CE_n(!base_ram_addr[20] || base_ram_ce_n),
        .WE_n(!base_ram_addr[20] || base_ram_we_n),
        .LB_n(base_ram_be_n[2]),
        .UB_n(base_ram_be_n[3])
    );

    initial begin
        reg [7:0] tmp_array[0:1048575];
        integer n_File_ID, n_Init_Size;
        n_File_ID = $fopen("../../../../../firmware/kernel.bin", "rb");
        if (!n_File_ID) begin
            n_Init_Size = 0;
            $display("Failed to open BaseRAM init file");
        end else begin
            n_Init_Size = $fread(tmp_array, n_File_ID);
            $fclose(n_File_ID);
        end
        $display("BaseRAM Init Size(bytes): %d", n_Init_Size);
        for (integer i = 0; i < 1048576; i++) begin
            low0.mem_array0[i] = 0;
            low0.mem_array1[i] = 0;
            low1.mem_array0[i] = 0;
            low1.mem_array1[i] = 0;
            high0.mem_array0[i] = 0;
            high0.mem_array1[i] = 0;
            high1.mem_array0[i] = 0;
            high1.mem_array1[i] = 0;
        end
        for (integer i = 0; i < n_Init_Size; i++) begin
            case (i % 4)
                0: low0.mem_array0[i/4] = tmp_array[i];
                1: low0.mem_array1[i/4] = tmp_array[i];
                2: low1.mem_array0[i/4] = tmp_array[i];
                3: low1.mem_array1[i/4] = tmp_array[i];
            endcase
        end
    end
endmodule
