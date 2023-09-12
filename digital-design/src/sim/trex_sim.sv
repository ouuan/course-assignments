`timescale 1ns / 1ps

module trex_sim;
    import trex_pkg::*;
    
    logic clk;
    logic rst;
    logic[5:0] timer = 0;
    logic[3:0] speed = 6;
    logic jump = 0;
    logic crash = 0;
    logic[9:0] x_pos;
    logic[9:0] y_pos;
    logic[2:0] frame;
    
    trex dut (
        .clk,
        .rst,
        .timer,
        .speed,
        .jump,
        .crash,
        .x_pos,
        .y_pos,
        .frame
    );

    integer counter = 0;

    initial begin
        forever begin
            #10 clk = 1;
            counter = counter + 1;
            if (counter == CLK_PER_FRAME) begin
                counter = 0;
                timer = timer + 1;
                if (timer == FPS) begin
                    timer = 0;
                end
            end
            #10 clk = 0;
        end
    end

    initial begin
        #100 rst = 1;
        #100 rst = 0;
        speed = 6;
        #100 jump = 1;
        #100 jump = 0;
        #1000000000 crash = 1;
        #100 $finish;
    end

endmodule
