module horizon_sim;
    import horizon_pkg::*;

    import obstacle_pkg::frame_t;

    import collision_pkg::*;

    parameter FPS = 60;
    parameter RANDOM_SEED = 19260817;

    logic clk = 0;
    logic rst = 0;

    logic update;
    logic[5:0] timer = 0;

    logic start = 0;
    logic crash = 0;

    logic[14:0] speed = 6 * 1024;

    logic signed[10:0] obstacle_x_pos[MAX_OBSTACLES];
    logic[9:0] obstacle_y_pos[MAX_OBSTACLES];
    logic[9:0] obstacle_width[MAX_OBSTACLES];
    logic[9:0] obstacle_height[MAX_OBSTACLES];
    frame_t obstacle_frame[MAX_OBSTACLES];

    collision_pkg::collision_box_t
        collision_box[obstacle_pkg::COLLISION_BOX_COUNT];

    logic rng_load = 1;
    logic[10:0] rng_data;
    lfsr_prng #(
        .DATA_WIDTH(11),
        .INVERT(0)
    ) prng_gap (
        .clk(clk),
        .load(rng_load),
        .seed(RANDOM_SEED),

        .enable(1),
        .data_out(rng_data)
    );

    horizon dut (
        .clk,
        .rst,
        .update,
        .timer,
        .start,
        .crash,
        .rng_data,
        .has_obstacles(1),
        .speed,
        .obstacle_x_pos,
        .obstacle_y_pos,
        .obstacle_width,
        .obstacle_height,
        .obstacle_frame,
        .collision_box
    );

    assign update = timer % 7 == 0;

    initial begin
        forever begin
            #5 clk = 0;
            timer = timer + 1;
            if (timer == FPS) begin
                timer = 0;
            end
            #5 clk = 1;
        end
    end

    initial begin
        #100 rst = 1; rng_load = 1;
        #100 rst = 0; rng_load = 0;
        #100 start = 1;
        #10 start = 0;
        #1000 speed = 9;
        #100000 crash = 1;
        #100 $finish;
    end

endmodule;
