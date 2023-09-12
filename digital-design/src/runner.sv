package runner_pkg;    
    typedef enum {
        WAITING,
        RUNNING,
        CRASHED,
        RESTARTING
    } state_t;

    typedef enum {
        CACTUS_LARGE,
        CACTUS_SMALL,
        CLOUD,
        HORIZON,
        MOON,
        PTERODACTYL,
        DISTANCE,
        HIGH_SCORE,
        GAME_OVER,
        TREX,
        STAR,
        LIFE,
        MULTIPLY,
        LIFE_NUM
    } element_t;

    typedef struct packed {
        logic[11:0] x;
        logic[11:0] y;
        logic[11:0] w;
        logic[11:0] h;
    } sprite_t;

    typedef struct packed {
        logic signed[11:0] x;
        logic signed[11:0] y;
    } pos_t;

    parameter CLK_FREQ = 33_333_333;
    parameter FPS = 60;
    parameter CLK_PER_FRAME = CLK_FREQ / FPS;

    parameter LIFES = 9;

    parameter CLEAR_TIME = 3 * FPS;

    parameter RESTART_TIME = FPS;

    // Scale from pixel speed to game speed
    parameter SPEED_SCALE = 1024;

    parameter SPEED = 6 * SPEED_SCALE;
    parameter SLOW_SPEED = 4 * SPEED_SCALE;

    parameter MAX_SPEED = 13 * SPEED_SCALE;
    parameter SLOW_MAX_SPEED = 9 * SPEED_SCALE;

    parameter ACCELERATION = 1;

    parameter GAME_WIDTH = 640;
    parameter GAME_HEIGHT = 150;

    parameter INVERT_FADE_DURATION = 720;
    parameter MAX_NIGHT_RATE = 63;
    parameter NIGHT_RATE_DELTA = 1;

    parameter ELEMENT_TYPES = 14;

    parameter RENDER_SLOTS = 37;
    parameter int RENDER_INDEX[ELEMENT_TYPES] = '{
        CACTUS_LARGE: 11,
        CACTUS_SMALL: 11,
        CLOUD: 4,
        HORIZON: 0,
        MOON: 10,
        PTERODACTYL: 11,
        DISTANCE: 19,
        HIGH_SCORE: 24,
        // This slot is shared with the space in "HI XXXXX"
        GAME_OVER: 26,
        TREX: 18,
        STAR: 2,
        LIFE: 32,
        MULTIPLY: 33,
        LIFE_NUM: 34
    };

    // (x, y)
    parameter int SPRITE[ELEMENT_TYPES][2] = '{
        CACTUS_LARGE: '{652, 2},
        CACTUS_SMALL: '{446, 2},
        CLOUD: '{166, 2},
        HORIZON: '{2, 104},
        MOON: '{954, 2},
        PTERODACTYL: '{260, 2},
        DISTANCE: '{1294, 2},
        HIGH_SCORE: '{1294, 60},
        GAME_OVER: '{1294, 28},
        TREX: '{1678, 2},
        STAR: '{1276, 2},
        LIFE: '{1540, 56},
        MULTIPLY: '{1571, 56},
        LIFE_NUM: '{1294, 2}
    };

    import trex_pkg::WAITING0;
    import trex_pkg::WAITING1;
    import trex_pkg::RUNNING0;
    import trex_pkg::RUNNING1;
    import trex_pkg::JUMPING0;
    import trex_pkg::DUCKING0;
    import trex_pkg::DUCKING1;
    import trex_pkg::CRASHED0;

    parameter int SPRITE_TREX_OFFSET[8] = '{
        WAITING0: 0,
        WAITING1: 88,
        RUNNING0: 176,
        RUNNING1: 264,
        JUMPING0: 0,
        DUCKING0: 528,
        DUCKING1: 646,
        CRASHED0: 440
    };

    parameter int SPRITE_HORIZON_LINE_OFFSET[2] = '{0, 1120};

    import night_pkg::NUM_PHASES;

    parameter int SPRITE_MOON_OFFSET[NUM_PHASES] = '{
        280, 240, 200, 120, 80, 40, 0
    };

    import obstacle_pkg::NONE_0;
    import obstacle_pkg::CACTUS_SMALL_0;
    import obstacle_pkg::CACTUS_LARGE_0;
    import obstacle_pkg::PTERODACTYL_0;
    import obstacle_pkg::PTERODACTYL_1;

    // Offset of specific obstacles for each size.
    // Multiply by 2 for high DPI.
    parameter int SPRITE_OBSTACLE_OFFSET[5][3] = '{
        NONE_0: '{0, 0, 0},
        CACTUS_SMALL_0: '{
            0,
            17 * 1 * 2,
            17 * 3 * 2
        },
        CACTUS_LARGE_0: '{
            0,
            25 * 1 * 2,
            25 * 3 * 2
        },
        PTERODACTYL_0: '{0, 0, 0},
        PTERODACTYL_1: '{
            46 * 2,
            0,
            0
        }
    };

    parameter GAME_OVER_WIDTH = 191;
    parameter GAME_OVER_HEIGHT = 11;
    parameter GAME_OVER_X = GAME_WIDTH / 2 - GAME_OVER_WIDTH / 2;
    parameter GAME_OVER_Y = (GAME_HEIGHT - 25) / 3;

    parameter LIFE_WIDTH = 31;
    parameter LIFE_HEIGHT = 28;
    parameter LIFE_X = 22;
    parameter LIFE_Y = 16;

    parameter MULTIPLY_WIDTH = 22;
    parameter MULTIPLY_HEIGHT = LIFE_HEIGHT;
    parameter MULTIPLY_X = LIFE_X + LIFE_WIDTH;
    parameter MULTIPLY_Y = LIFE_Y;

    parameter LIFE_NUM_X = MULTIPLY_X + MULTIPLY_WIDTH;
    parameter LIFE_NUM_Y = distance_meter_pkg::Y;

    parameter TREX_BOX_COUNT = trex_pkg::COLLISION_BOX_COUNT;
    parameter OBSTACLE_BOX_COUNT = obstacle_pkg::COLLISION_BOX_COUNT;

endpackage

import runner_pkg::RENDER_SLOTS;
import runner_pkg::sprite_t;
import runner_pkg::pos_t;

module runner (
    input clk,
    input rst,   

    // Sensor mode.
    input sensor_mode,

    input jumping,
    input ducking,

    input painter_finished,

    input[10:0] random_seed,

    output sprite_t sprite[RENDER_SLOTS],
    output pos_t pos[RENDER_SLOTS],

    output logic[5:0] night_rate
);
    import runner_pkg::*;

    import collision_pkg::*;
    import distance_meter_pkg::MAX_DISTANCE_UNITS;
    import distance_meter_pkg::MAX_HIGH_SCORE_UNITS;
    import horizon_pkg::MAX_CLOUDS;
    import horizon_pkg::MAX_OBSTACLES;

    state_t state, next_state;

    // Always use normal mode.
    logic slow = 0;

    logic update;

    logic[3:0] life;

    logic[5:0] timer;
    logic[14:0] speed;

    // Generate obstacles only after CLEAR_TIME.
    logic[7:0] clear_timer;
    logic has_obstacles;

    // Only allow restart after RESTART_TIME.
    logic[7:0] restart_timer;

    logic start;
    logic restart;

    logic crashed;

    logic jumping_last;

    logic inverted;
    logic invert_trigger;
    logic[9:0] invert_timer;

    logic painter_finished_last;

    logic rng_load;
    bit[10:0] rng_data, rng_data2;
    lfsr_prng #(
        .DATA_WIDTH(11),
        .INVERT(0)
    ) prng_gap (
        .clk(clk),
        .load(rng_load),
        .seed(random_seed + timer),

        .enable(1),
        .data_out(rng_data)
    );
    lfsr_prng #(
        .DATA_WIDTH(11),
        .INVERT(0)
    ) prng_gap2 (
        .clk(clk),
        .load(rng_load),
        .seed(random_seed + timer + 1),

        .enable(1),
        .data_out(rng_data2)
    );

    logic trex_immune;
    logic signed[11:0] trex_x_pos;
    logic signed[11:0] trex_y_pos;
    logic[9:0] trex_width;
    logic[9:0] trex_height;
    trex_pkg::frame_t trex_frame;
    logic trex_paint;

    collision_box_t trex_box[TREX_BOX_COUNT];

    trex trex_inst (
        .clk,
        .rst(rst || restart),

        .update,
        .timer,
        .speed(speed / SPEED_SCALE),

        .slow,

        .jump(jumping),
        .duck(ducking),
        .crack(crashed),
        // Life is going to 0, dead
        .crash(crashed && !trex_immune && life == 1),

        .immune(trex_immune),

        .x_pos(trex_x_pos),
        .y_pos(trex_y_pos),
        .width(trex_width),
        .height(trex_height),
        .frame(trex_frame),

        .paint(trex_paint),

        .collision_box(trex_box)
    );

    logic[3:0] distance_meter_digits[MAX_DISTANCE_UNITS];
    logic[3:0] distance_meter_high_score[MAX_HIGH_SCORE_UNITS];
    logic distance_meter_paint;

    distance_meter distance_meter_inst (
        .clk,
        .rst(rst),

        .update,
        .restart,
        .speed(state == CRASHED ? 0 : speed),

        .slow,

        .digits(distance_meter_digits),
        .high_score(distance_meter_high_score),
        .paint(distance_meter_paint),

        .invert_trigger
    );

    logic signed[10:0] horizon_line_x_pos[2];

    logic horizon_line_bump[2];

    logic signed[10:0] moon_x_pos;
    logic[9:0] moon_width;
    logic[2:0] moon_phase;

    logic signed[10:0] star_x_pos[NUM_STARS];
    logic[9:0] star_y_pos[NUM_STARS];

    logic night_paint;

    logic cloud_start[MAX_CLOUDS];

    logic signed[10:0] cloud_x_pos[MAX_CLOUDS];
    logic[9:0] cloud_y_pos[MAX_CLOUDS];

    logic obstacle_start[MAX_OBSTACLES];

    logic signed[10:0] obstacle_x_pos[MAX_OBSTACLES];
    logic[9:0] obstacle_y_pos[MAX_OBSTACLES];
    logic[9:0] obstacle_width[MAX_OBSTACLES];
    logic[9:0] obstacle_height[MAX_OBSTACLES];
    logic[1:0] obstacle_size[MAX_OBSTACLES];

    obstacle_pkg::frame_t obstacle_frame[MAX_OBSTACLES];

    collision_box_t obstacle_box[OBSTACLE_BOX_COUNT];

    horizon horizon_inst (
        .clk,
        .rst(rst || restart),

        .update,
        .timer,

        .start,
        .crash(state == CRASHED),

        .speed,

        .slow,

        .rng_data('{rng_data, rng_data2}),

        .has_obstacles,

        .night_rate,

        .horizon_line_x_pos,

        .horizon_line_bump,

        .moon_x_pos,
        .moon_width,
        .moon_phase,

        .star_x_pos,
        .star_y_pos,

        .night_paint,

        .cloud_start,

        .cloud_x_pos,
        .cloud_y_pos,

        .obstacle_start,

        .obstacle_x_pos,
        .obstacle_y_pos,
        .obstacle_width,
        .obstacle_height,
        .obstacle_size,

        .obstacle_frame,

        .collision_box(obstacle_box)
    );

    always_comb begin
        case (state)
            WAITING: begin
                next_state = update && jumping ? RUNNING : WAITING;
            end
            RUNNING: begin
                if (crashed) begin
                    next_state = life > 0 ? RUNNING : CRASHED;
                end else begin
                    next_state = RUNNING;
                end
            end
            CRASHED: begin
                // Jump again after crash to restart.
                if (restart_timer == RESTART_TIME &&
                    jumping && !jumping_last
                ) begin
                    next_state = RESTARTING;
                end else begin
                    next_state = CRASHED;
                end
            end
            RESTARTING: begin
                // Return to waiting status after restart.
                if (!jumping) begin
                    next_state = WAITING;
                end else begin
                    next_state = RESTARTING;
                end
            end
        endcase
    end

    always_ff @(posedge clk) begin
        if (rst) begin
            state <= WAITING;
            life <= 0;
            update <= 0;
            start <= 0;
            restart <= 0;
            crashed <= 0;
            timer <= 0;
            jumping_last <= 0;
            painter_finished_last <= 0;
            speed <= 0;
            clear_timer <= 0;
            restart_timer <= 0;
            has_obstacles <= 0;
            rng_load <= 1;
            inverted <= 0;
            invert_timer <= 0;
            night_rate <= 0;
        end else begin
            state <= next_state;

            jumping_last <= jumping;
            painter_finished_last <= painter_finished;

            crashed <= check_for_collision();

            if (update) begin
                update_night_rate();
            end

            // Posedge of painter_finished, step game loop
            if (painter_finished && !painter_finished_last) begin
                update <= 1;
                if (timer + 1 == FPS) begin
                    timer <= 0;
                end else begin
                    timer <= timer + 1;
                end

                if (next_state == CRASHED) begin
                    if (restart_timer < RESTART_TIME) begin
                        restart_timer <= restart_timer + 1;
                    end
                end
            end else begin
                update <= 0;
            end

            if (next_state == RESTARTING) begin
                restart <= 1;
                reset();
            end else begin
                restart <= 0;
            end
            
            if (update && next_state == RUNNING) begin
                rng_load <= 0;
                if (state == WAITING) begin
                    init();
                end else begin
                    run();
                end
            end
        end
    end

    task update_night_rate;
        // Update night rate.
        if (inverted && night_rate < MAX_NIGHT_RATE) begin
            night_rate <= night_rate + NIGHT_RATE_DELTA;
        end else if (!inverted && night_rate > 0) begin
            night_rate <= night_rate - NIGHT_RATE_DELTA;
        end
    endtask

    task reset;
        life <= 0;
        start <= 0;
        speed <= 0;
        clear_timer <= 0;
        restart_timer <= 0;
        has_obstacles <= 0;
        inverted <= 0;
        night_rate <= 0;
    endtask

    task init;
        // Only one life if button mode.
        life <= sensor_mode ? LIFES : 1;
        start <= 1;
        speed <= slow ? SLOW_SPEED : SPEED;
    endtask

    task run;
        if (clear_timer < CLEAR_TIME) begin
            clear_timer <= clear_timer + 1;
        end

        // Stop generating obstacles after crack.
        if (trex_immune) begin
            has_obstacles <= 0;
        end else if (clear_timer == CLEAR_TIME) begin
            has_obstacles <= 1;
        end

        // Damage after crack.
        if (crashed && !trex_immune) begin
            life <= life - 1;
        end

        if (speed + ACCELERATION <= (slow ? SLOW_MAX_SPEED : MAX_SPEED)) begin
            // In slow mode, half the acceleration.
            if (!slow || slow && timer % 2 == 0) begin
                speed <= speed + ACCELERATION;
            end
        end
        
        // Night mode trigger.
        if (invert_timer == INVERT_FADE_DURATION) begin
            invert_timer <= 0;
            inverted <= 0;
        end else if (inverted) begin
            invert_timer <= invert_timer + 1;
        end else if (invert_trigger) begin
            inverted <= 1;
        end
    endtask

    // Collision check.
    function logic check_for_collision;
        automatic logic crashed = 0;
        for (int i = 0; i < TREX_BOX_COUNT; i++) begin
            for (int j = 0; j < OBSTACLE_BOX_COUNT; j++) begin
                crashed |= box_compare(trex_box[i], obstacle_box[j]);
            end
        end
        return crashed;
    endfunction

    // Get the corresponding sprite type for a obstacle frame.
    function element_t element_type(obstacle_pkg::frame_t frame);
        case (frame)
            obstacle_pkg::NONE_0: return CACTUS_SMALL;
            obstacle_pkg::CACTUS_SMALL_0: return CACTUS_SMALL;
            obstacle_pkg::CACTUS_LARGE_0: return CACTUS_LARGE;
            obstacle_pkg::PTERODACTYL_0: return PTERODACTYL;
            obstacle_pkg::PTERODACTYL_1: return PTERODACTYL;
        endcase
    endfunction

    // Sprite output. Multiply by 2 for high DPI.
    always_ff @(posedge clk) begin
        for (int i = 0; i < RENDER_SLOTS; i++) begin
            sprite[i] = '{0, 0, 0, 0};
            pos[i] = '{0, 0};
        end

        if (!rst) begin
            // T-Rex
            if (trex_paint) begin
                sprite[RENDER_INDEX[TREX]] <= '{
                    SPRITE[TREX][0] + SPRITE_TREX_OFFSET[trex_frame],
                    SPRITE[TREX][1],
                    trex_width * 2,
                    trex_pkg::HEIGHT * 2
                };
                pos[RENDER_INDEX[TREX]] <= '{trex_x_pos * 2, trex_y_pos * 2};
            end

            // Horizon lines
            for (int i = 0; i < 2; i++) begin
                sprite[RENDER_INDEX[HORIZON] + i] <= '{
                    SPRITE[HORIZON][0]
                        + SPRITE_HORIZON_LINE_OFFSET[horizon_line_bump[i]],
                    SPRITE[HORIZON][1],
                    horizon_line_pkg::WIDTH * 2,
                    horizon_line_pkg::HEIGHT * 2
                };
                pos[RENDER_INDEX[HORIZON] + i] <= '{
                    horizon_line_x_pos[i] * 2,
                    horizon_line_pkg::Y_POS * 2
                };
            end

            // Moon
            if (night_paint) begin
                sprite[RENDER_INDEX[MOON]] <= '{
                    SPRITE[MOON][0] + SPRITE_MOON_OFFSET[moon_phase],
                    SPRITE[MOON][1],
                    moon_width * 2,
                    night_pkg::HEIGHT * 2
                };
                pos[RENDER_INDEX[MOON]] <= '{
                    moon_x_pos * 2,
                    night_pkg::MOON_Y_POS * 2
                };
            end

            // Stars
            if (night_paint) begin
                for (int i = 0; i < NUM_STARS; i++) begin
                    sprite[RENDER_INDEX[STAR] + i] <= '{
                        SPRITE[STAR][0],
                        SPRITE[STAR][1],
                        night_pkg::STAR_SIZE * 2,
                        night_pkg::STAR_SIZE * 2
                    };
                    pos[RENDER_INDEX[STAR] + i] <= '{
                        star_x_pos[i] * 2,
                        star_y_pos[i] * 2
                    };
                end
            end

            // Clouds
            for (int i = 0; i < MAX_CLOUDS; i++) begin
                if (cloud_start[i]) begin
                    sprite[RENDER_INDEX[CLOUD] + i] <= '{
                        SPRITE[CLOUD][0],
                        SPRITE[CLOUD][1],
                        cloud_pkg::WIDTH * 2,
                        cloud_pkg::HEIGHT * 2
                    };
                    pos[RENDER_INDEX[CLOUD] + i] <= '{
                        cloud_x_pos[i] * 2,
                        cloud_y_pos[i] * 2
                    };
                end
            end

            // Obstacles
            for (int i = 0; i < MAX_OBSTACLES; i++) begin
                if (obstacle_start[i]
                    && obstacle_frame[i] != obstacle_pkg::NONE_0
                ) begin
                    sprite[RENDER_INDEX[element_type(obstacle_frame[i])]
                        + i] <= '{
                        SPRITE[element_type(obstacle_frame[i])][0]
                            + SPRITE_OBSTACLE_OFFSET[obstacle_frame[i]]
                                [obstacle_size[i] - 1],
                        SPRITE[element_type(obstacle_frame[i])][1],
                        obstacle_width[i] * 2,
                        obstacle_height[i] * 2
                    };
                    pos[RENDER_INDEX[element_type(obstacle_frame[i])]
                        + i] <= '{
                        obstacle_x_pos[i] * 2,
                        obstacle_y_pos[i] * 2
                    };
                end
            end

            // Distance meter
            if (distance_meter_paint) begin
                for (int i = 0; i < MAX_DISTANCE_UNITS; i++) begin
                    sprite[RENDER_INDEX[DISTANCE] + i] <= '{
                        SPRITE[DISTANCE][0]
                            + distance_meter_digits[i]
                            * distance_meter_pkg::WIDTH * 2,
                        SPRITE[DISTANCE][1],
                        distance_meter_pkg::WIDTH * 2,
                        distance_meter_pkg::HEIGHT * 2
                    };
                    pos[RENDER_INDEX[DISTANCE] + i] <= '{
                        distance_meter_pkg::X
                            + i * distance_meter_pkg::DEST_WIDTH * 2,
                        distance_meter_pkg::Y
                    };
                end
            end

            // High score
            for (int i = 0; i < MAX_HIGH_SCORE_UNITS; i++) begin
                // Ignore the space
                if (i == 2) begin
                    continue;
                end
                sprite[RENDER_INDEX[HIGH_SCORE] + i] <= '{
                    SPRITE[HIGH_SCORE][0]
                        + distance_meter_high_score[i]
                        * distance_meter_pkg::WIDTH * 2,
                    SPRITE[HIGH_SCORE][1],
                    distance_meter_pkg::WIDTH * 2,
                    distance_meter_pkg::HEIGHT * 2
                };
                pos[RENDER_INDEX[HIGH_SCORE] + i] <= '{
                    distance_meter_pkg::HIGH_SCORE_X
                        + i * distance_meter_pkg::DEST_WIDTH * 2,
                    distance_meter_pkg::Y
                };
            end

            // Game over panel
            if (state == CRASHED) begin
                sprite[RENDER_INDEX[GAME_OVER]] <= '{
                    SPRITE[GAME_OVER][0],
                    SPRITE[GAME_OVER][1],
                    GAME_OVER_WIDTH * 2,
                    GAME_OVER_HEIGHT * 2
                };
                pos[RENDER_INDEX[GAME_OVER]] <= '{
                    GAME_OVER_X * 2,
                    GAME_OVER_Y * 2
                };
            end

            // Life. Note that this uses sprite coords directly without scaling
            if (sensor_mode && life > 5) begin
                sprite[RENDER_INDEX[LIFE]] <= '{
                    SPRITE[LIFE][0],
                    SPRITE[LIFE][1],
                    LIFE_WIDTH,
                    LIFE_HEIGHT
                };
                pos[RENDER_INDEX[LIFE]] <= '{
                    LIFE_X,
                    LIFE_Y
                };

                sprite[RENDER_INDEX[MULTIPLY]] <= '{
                    SPRITE[MULTIPLY][0],
                    SPRITE[MULTIPLY][1],
                    MULTIPLY_WIDTH,
                    MULTIPLY_HEIGHT
                };
                pos[RENDER_INDEX[MULTIPLY]] <= '{
                    MULTIPLY_X,
                    MULTIPLY_Y
                };

                sprite[RENDER_INDEX[LIFE_NUM]] <= '{
                    SPRITE[LIFE_NUM][0] + life * distance_meter_pkg::WIDTH * 2,
                    SPRITE[LIFE_NUM][1],
                    distance_meter_pkg::WIDTH * 2,
                    distance_meter_pkg::HEIGHT * 2
                };
                pos[RENDER_INDEX[LIFE_NUM]] <= '{
                    LIFE_NUM_X,
                    LIFE_NUM_Y
                };
            end else if (sensor_mode) begin
                // Show life icons directly when life <= 5
                for (int i = 0; i < 5; i++) begin
                    if (life > i) begin
                        sprite[RENDER_INDEX[LIFE] + i] <= '{
                            SPRITE[LIFE][0],
                            SPRITE[LIFE][1],
                            LIFE_WIDTH,
                            LIFE_HEIGHT
                        };
                        pos[RENDER_INDEX[LIFE] + i] <= '{
                            LIFE_X + i * LIFE_WIDTH,
                            LIFE_Y
                        };
                    end
                end
            end
        end
    end

endmodule
