package horizon_pkg;
    typedef enum { 
        WAITING,
        RUNNING,
        UPDATE_CLOUDS_0,
        UPDATE_CLOUDS_1,
        UPDATE_CLOUDS_2,
        UPDATE_OBSTACLES_0,
        UPDATE_OBSTACLES_1,
        UPDATE_OBSTACLES_2,
        CRASHED
    } state_t;

    parameter MAX_CLOUDS = 6;

    parameter MAX_OBSTACLES = 7;
    parameter MAX_OBSTACLE_DUPLICATION = 2;
    parameter OBSTACLE_TYPES = obstacle_pkg::TYPE_COUNT - 1;

    parameter GAME_WIDTH = 640;
    parameter SPEED_SCALE = 1024;
endpackage

import horizon_pkg::MAX_OBSTACLES;

// Horizon background class.
module horizon (
    input clk,
    input rst,

    input update,
    input[5:0] timer,

    input start,
    input crash,

    input[14:0] speed,

    input slow,

    input bit[10:0] rng_data[2],

    // Enable obstacle generation.
    input has_obstacles,

    input logic[5:0] night_rate,

    output logic signed[10:0] horizon_line_x_pos[2],

    output logic horizon_line_bump[2],

    output logic signed[10:0] moon_x_pos,
    output logic[9:0] moon_width,
    output logic[2:0] moon_phase,

    output logic signed[10:0] star_x_pos[NUM_STARS],
    output logic[9:0] star_y_pos[NUM_STARS],

    output logic night_paint,

    output logic cloud_start[MAX_CLOUDS],

    output logic signed[10:0] cloud_x_pos[MAX_CLOUDS],
    output logic[9:0] cloud_y_pos[MAX_CLOUDS],

    output logic obstacle_start[MAX_OBSTACLES],

    output logic signed[10:0] obstacle_x_pos[MAX_OBSTACLES],
    output logic[9:0] obstacle_y_pos[MAX_OBSTACLES],
    output logic[9:0] obstacle_width[MAX_OBSTACLES],
    output logic[9:0] obstacle_height[MAX_OBSTACLES],
    output logic[1:0] obstacle_size[MAX_OBSTACLES],

    output obstacle_pkg::frame_t obstacle_frame[MAX_OBSTACLES],

    output collision_pkg::collision_box_t
        collision_box[obstacle_pkg::COLLISION_BOX_COUNT]
);
    import horizon_pkg::*;

    import night_pkg::NUM_STARS;
    import night_pkg::SEGMENT_SIZE;
    import night_pkg::STAR_MAX_Y;

    import cloud_pkg::MIN_SKY_LEVEL;
    import cloud_pkg::MAX_SKY_LEVEL;
    import cloud_pkg::MIN_CLOUD_GAP;
    import cloud_pkg::MAX_CLOUD_GAP;

    import obstacle_pkg::MAX_OBSTACLE_LENGTH;
    import obstacle_pkg::MIN_SPEED;

    typedef obstacle_pkg::type_t obstacle_t;
    typedef obstacle_pkg::frame_t obstacle_frame_t;
    
    state_t state, next_state;

    // Horizon line.
    horizon_line horizon_line_inst (
        .clk,
        .rst,

        .update,
        .speed,

        .start,
        .crash,

        .rng_data(rng_data[0]),

        .x_pos(horizon_line_x_pos),

        .bump(horizon_line_bump)
    );

    // Night elements.
    logic[9:0] star_x_rand[NUM_STARS];
    logic[9:0] star_y_rand[NUM_STARS];

    genvar i1;
    generate
        for (i1 = 0; i1 < NUM_STARS; i1++) begin: div_night
            div div_star_x (
                .clock(clk),
                .numer(rng_data[i1]),
                .denom(SEGMENT_SIZE),
                .remain(star_x_rand[i1])
            );
            div div_star_y (
                .clock(clk),
                .numer(rng_data[i1]),
                .denom(STAR_MAX_Y),
                .remain(star_y_rand[i1])
            );
        end
    endgenerate

    night night_inst (
        .clk,
        .rst,

        .update,

        .crash,

        .star_x_rand,
        .star_y_rand,

        .night_rate,

        .moon_x_pos,
        .moon_width,
        .moon_phase,

        .star_x_pos,
        .star_y_pos,

        .activated(night_paint)
    );

    // Cloud queue.
    logic[2:0] cloud_front;
    logic[2:0] cloud_back;

    logic cloud_update;

    logic cloud_remove[MAX_CLOUDS];
    logic[10:0] cloud_gap[MAX_CLOUDS];
    logic cloud_visible[MAX_CLOUDS];

    // Random number for cloud level and gap.
    logic[9:0] cloud_level_rand;
    logic[10:0] cloud_gap_rand;

    div div_cloud_level (
        .clock(clk),
        .numer(rng_data[0]),
        .denom(MIN_SKY_LEVEL - MAX_SKY_LEVEL),
        .remain(cloud_level_rand)
    );
    div div_cloud_gap (
        .clock(clk),
        .numer(rng_data[0]),
        .denom(MAX_CLOUD_GAP - MIN_CLOUD_GAP),
        .remain(cloud_gap_rand)
    );

    genvar i0;
    generate
        for (i0 = 0; i0 < MAX_CLOUDS; i0++) begin: clouds
            cloud cloud_inst (
                .clk,
                .rst,

                .update(cloud_update),
                .speed,

                .start(cloud_start[i0]),
                .crash,

                .level_rand(cloud_level_rand),
                .gap_rand(cloud_gap_rand),

                .remove(cloud_remove[i0]),
                .gap(cloud_gap[i0]),
                .visible(cloud_visible[i0]),

                .x_pos(cloud_x_pos[i0]),
                .y_pos(cloud_y_pos[i0])
            );
        end
    endgenerate

    // Obstacle queue.
    logic[2:0] obstacle_front;
    logic[2:0] obstacle_back;

    logic obstacle_update;

    obstacle_t obstacle_type[MAX_OBSTACLES];
    logic obstacle_remove[MAX_OBSTACLES];
    logic[10:0] obstacle_gap[MAX_OBSTACLES];
    logic obstacle_visible[MAX_OBSTACLES];

    collision_pkg::collision_box_t
        obstacle_box[MAX_OBSTACLES][obstacle_pkg::COLLISION_BOX_COUNT];

    // Shared divider.
    logic[10:0] div_obstacle_gap_denom[MAX_OBSTACLES];
    logic[10:0] div_obstacle_gap_remain;

    div div_obstacle_gap (
        .clock(clk),
        .numer(rng_data[0]),
        .denom(div_obstacle_gap_denom[last_obstacle()]),
        .remain(div_obstacle_gap_remain)
    );

    genvar i;
    generate
        for (i = 0; i < MAX_OBSTACLES; i++) begin: obstacles
            obstacle obstacle_inst (
                .clk,
                .rst, 

                .update(obstacle_update),
                .timer,
                .speed,

                .slow,

                .typ(obstacle_type[i]),
                .start(obstacle_start[i]),
                .crash,

                .div_remain(div_obstacle_gap_remain),
                
                .remove(obstacle_remove[i]),
                .gap(obstacle_gap[i]),
                .visible(obstacle_visible[i]),

                .x_pos(obstacle_x_pos[i]),
                .y_pos(obstacle_y_pos[i]),
                .width(obstacle_width[i]),
                .height(obstacle_height[i]),
                .size(obstacle_size[i]),

                .frame(obstacle_frame[i]),

                .collision_box(obstacle_box[i]),

                .div_denom(div_obstacle_gap_denom[i])
            );
        end
    endgenerate

    always_comb begin
        case (state)
            WAITING: begin
                next_state = start ? RUNNING : WAITING;
            end
            RUNNING: begin
                if (crash) begin
                    next_state = CRASHED;
                end else begin
                    next_state = update ? UPDATE_CLOUDS_0 : RUNNING;
                end
            end
            UPDATE_CLOUDS_0: begin
                next_state = UPDATE_CLOUDS_1;
            end
            UPDATE_CLOUDS_1: begin
                next_state = UPDATE_CLOUDS_2;
            end
            UPDATE_CLOUDS_2: begin
                next_state = UPDATE_OBSTACLES_0;
            end
            UPDATE_OBSTACLES_0: begin
                next_state = UPDATE_OBSTACLES_1;
            end
            UPDATE_OBSTACLES_1: begin
                next_state = UPDATE_OBSTACLES_2;
            end
            UPDATE_OBSTACLES_2: begin
                next_state = RUNNING;
            end
            CRASHED: begin
                next_state = CRASHED;
            end
            default: begin
                next_state = WAITING;
            end
        endcase
    end

    always_ff @(posedge clk) begin
        if (rst) begin
            state <= WAITING;
            cloud_update <= 0;
            cloud_front <= 0;
            cloud_back <= 0;
            for (int i = 0; i < MAX_CLOUDS; i++) begin
                cloud_start[i] <= 0;
            end
            obstacle_update <= 0;
            obstacle_front <= 0;
            obstacle_back <= 0;
            for (int i = 0; i < MAX_OBSTACLES; i++) begin
                obstacle_start[i] <= 0;
                obstacle_type[i] <= obstacle_pkg::NONE;
            end
        end else begin
            state <= next_state;

            // Collision box for the leftmost obstacle.
            collision_box <= obstacle_box[obstacle_front];

            case (next_state)
                RUNNING: begin
                    obstacle_update <= 0;
                end
                UPDATE_CLOUDS_0: begin
                    update_clouds_0();
                end
                UPDATE_CLOUDS_1: begin
                    update_clouds_1();
                end
                UPDATE_CLOUDS_2: begin
                    update_clouds_2();
                end
                UPDATE_OBSTACLES_0: begin
                    update_obstacles_0();
                end
                UPDATE_OBSTACLES_1: begin
                    update_obstacles_1();
                end
                UPDATE_OBSTACLES_2: begin
                    update_obstacles_2();
                end
            endcase
        end
    end

    // Mod increment.
    function logic[2:0] incr(logic[2:0] x, int size);
        return x + 1 >= size ? 0 : x + 1;
    endfunction

    // Mod decrement.
    function logic[2:0] decr(logic[2:0] x, int size);
        return !x ? size - 1 : x - 1;
    endfunction

    // Last element in the clouds queue.
    function logic[2:0] last_cloud;
        return decr(cloud_back, MAX_CLOUDS);
    endfunction

    // Last element in the obstacle queue.
    function logic[2:0] last_obstacle;
        return decr(obstacle_back, MAX_OBSTACLES);
    endfunction

    // Update existing clouds and create new ones.
    task update_clouds_0;
        cloud_update <= 1;
        if (cloud_front == cloud_back) begin
            add_new_cloud();
        end else begin
            // Add new cloud if gap is large enough.
            // Cloud frequency slightly controled.
            if (rng_data[0][0]
                && cloud_visible[last_cloud()] && 15'sb0
                + cloud_x_pos[last_cloud()]
                + $signed(cloud_pkg::WIDTH)
                + $signed(12'(cloud_gap[last_cloud()]))
                < $signed(GAME_WIDTH)
            ) begin
                add_new_cloud();
            end
        end
    endtask

    // Wait for clouds to be updated.
    task update_clouds_1;
        cloud_update <= 0;
    endtask

    // Remove invisible clouds.
    task update_clouds_2;
        if (cloud_remove[cloud_front]) begin
            cloud_front <= incr(cloud_front, MAX_CLOUDS);
            cloud_start[cloud_front] <= 0;
        end
    endtask

    // Update existing obstacles and create new ones.
    task update_obstacles_0;
        obstacle_update <= 1;

        // Prevent obstacles from being generated.
        if (!has_obstacles) begin
            return;
        end

        if (obstacle_front == obstacle_back) begin
            add_new_obstacle();
        end else begin
            // Add new obstacle if gap is large enough. 
            if (obstacle_visible[last_obstacle()] && 15'sb0
                + obstacle_x_pos[last_obstacle()]
                + $signed(obstacle_width[last_obstacle()])
                + $signed(12'(obstacle_gap[last_obstacle()]))
                < $signed(GAME_WIDTH)
            ) begin
                add_new_obstacle();
            end
        end
    endtask

    // Wait for obstacles to be updated.
    task update_obstacles_1;
        obstacle_update <= 0;
    endtask

    // Remove invisible obstacles.
    task update_obstacles_2;
        if (obstacle_remove[obstacle_front]) begin
            obstacle_front <= incr(obstacle_front, MAX_OBSTACLES);
            obstacle_start[obstacle_front] <= 0;
        end
    endtask

    // Add a new cloud to the queue.
    task add_new_cloud;
        cloud_back <= incr(cloud_back, MAX_CLOUDS);
        cloud_start[cloud_back] <= 1;
    endtask

    // Returns whether the previous two obstacles are the same as the next one.
    function logic duplicate_obstacle_check(obstacle_t typ);
        automatic int duplicate_count = 0;
        automatic int now = last_obstacle();
        for (int i = 0;
             i < MAX_OBSTACLE_DUPLICATION;
             i++, now = decr(now, MAX_OBSTACLES)
        ) begin
            if (obstacle_type[now] == typ) begin
                duplicate_count++;
            end
        end
        return duplicate_count >= MAX_OBSTACLE_DUPLICATION;
    endfunction

    // Randomly pick a new obstacle type.
    function obstacle_t new_obstacle_type;
        automatic obstacle_t typ;
        // Try another type if duplicate or not allowed at current speed.
        for (int i = 0; i < OBSTACLE_TYPES; i++) begin
            typ = obstacle_t'(
                (rng_data[0] + i) % OBSTACLE_TYPES + 1
            );
            if (!duplicate_obstacle_check(typ)
                && speed >= MIN_SPEED[typ]
            ) begin
                return typ;
            end
        end
        return obstacle_pkg::NONE;
    endfunction

    // Add a new obstacle to the queue.
    task add_new_obstacle;
        obstacle_back <= incr(obstacle_back, MAX_OBSTACLES);
        obstacle_type[obstacle_back] <= new_obstacle_type();
        obstacle_start[obstacle_back] <= 1;
    endtask

endmodule
