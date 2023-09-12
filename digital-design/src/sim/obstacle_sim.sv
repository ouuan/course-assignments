`timescale 1ns / 1ps

module obstacle_sim;
    import obstacle_pkg::*;

    import collision_pkg::*;

    parameter FPS = 60;
    parameter CLK_PER_FRAME = 10;
    
    logic clk;
    logic rst;

    logic update;
    logic[5:0] timer = 0;
    logic[14:0] speed = 6 * 1024;

    type_t typ = NONE;
    logic start = 0;
    logic crash = 0;

    bit[10:0] rng_data;

    logic remove;
    logic[10:0] gap;
    logic visible;

    logic signed[10:0] x_pos;
    logic[9:0] y_pos;
    logic[9:0] width;
    logic[9:0] height;

    logic[1:0] size = 0;

    frame_t frame;

    collision_box_t collision_box[COLLISION_BOX_COUNT];
    
    obstacle dut (
        .clk,
        .rst,
        .update,
        .timer,
        .speed,
        .typ,
        .start,
        .crash,
        .rng_data,
        .remove,
        .gap,
        .visible,
        .x_pos,
        .y_pos,
        .width,
        .height,
        .size,
        .frame,
        .collision_box
    );

    integer counter = 0;

    initial begin
        forever begin
            #5 clk = 1;
            counter = counter + 1;
            rng_data = $random;
            if (counter == CLK_PER_FRAME) begin
                counter = 0;
                update = 1;
                timer = timer + 1;
                if (timer == FPS) begin
                    timer = 0;
                end
            end else begin
                update = 0;
            end
            #5 clk = 0;
        end
    end

    initial begin
        #100 rst = 1;
        #100 rst = 0;
        #100;
        crash = 0;

        typ = CACTUS_LARGE;
        size = 2;
        start = 1;
        #10000;

        typ = CACTUS_SMALL;
        size = 2;
        start = 1;
        #10000;

        #1000;
        typ = PTERODACTYL;
        size = 2;
        start = 1;
        #10000;

        $finish;
    end

endmodule
