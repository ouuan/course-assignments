`timescale 1ps / 1ps

module tb_frame_datapath
#(
    parameter DATA_WIDTH = 64,
    parameter ID_WIDTH = 3
)
(
    
);

    reg reset;
    reg [15:0] dip_sw;
    reg push_btn;
    reg [3:0] touch_btn;

    initial begin
        touch_btn = 0;
        push_btn = 0;
        reset = 1;
        #6000
        reset = 0;
        #20000
        dip_sw = 16'b0001000000100100;
        #20000
        push_btn = 1;
        #20000
        push_btn = 0;
        #20000
        dip_sw = 16'b0001000100100101;
        #20000
        push_btn = 1;
        #20000
        push_btn = 0;
        #20000
        dip_sw = 16'b0001001000100110;
        #20000
        push_btn = 1;
        #20000
        push_btn = 0;
        #20000
        dip_sw = 16'b0001001100100111;
        #20000
        push_btn = 1;
        #20000
        push_btn = 0;
    end

    wire clk_125M;

    clock clock_i(
        .clk_125M(clk_125M)
    );

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

    wire [15:0] led;

    // README: Instantiate your datapath.
    frame_datapath dut(
        .eth_clk(clk_125M),
        .reset(reset),

        .BTN(push_btn),
        .dip_sw(dip_sw),
        .touch_btn(touch_btn),
        .led(led),

        .s_data(in_data),
        .s_keep(in_keep),
        .s_last(in_last),
        .s_user(in_user),
        .s_id(in_id),
        .s_valid(in_valid),
        .s_ready(in_ready),

        .m_data(out_data),
        .m_keep(out_keep),
        .m_last(out_last),
        .m_user(out_user),
        .m_dest(out_dest),
        .m_valid(out_valid),
        .m_ready(out_ready)
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
endmodule
