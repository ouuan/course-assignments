`timescale 1ns / 1ps

parameter CLK_FREQ = 100_000_000;
parameter FPS = 60;
parameter CLK_PER_FRAME = CLK_FREQ / FPS;

module distance_meter_sim;

    import distance_meter_pkg::*;

    logic clk = 0;
    logic rst = 0;

    int counter = 0;
    int timer = 0;

    logic timer_pulse;

    logic[3:0] speed = 0;

    logic[3:0] digits[MAX_DISTANCE_UNITS];
    logic paint;

    distance_meter dut (
        .clk,
        .rst,
        .timer_pulse,
        .speed,
        .digits,
        .paint
    );

    // For faster simulation
    assign timer_pulse = clk;

    initial begin
        forever begin
            #5 clk = 1;
            // counter = counter + 1;
            // if (counter == CLK_PER_FRAME) begin
            //     counter = 0;
            //     timer_pulse = 1;
            //     timer = timer + 1;
            //     if (timer == FPS) begin
            //         timer = 0;
            //     end
            // end else begin
            //     timer_pulse = 0;
            // end
            #5 clk = 0;
        end
    end

    initial begin
        #100 rst = 1;
        #100 rst = 0;
        #100 speed = 6;

        // Crashed
        #14600 speed = 0;
        #1000 $finish;
    end

endmodule
