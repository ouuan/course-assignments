package trex_pkg;
    typedef enum logic[2:0] {
        WAITING0, WAITING1,
        RUNNING0, RUNNING1,
        JUMPING0,
        DUCKING0, DUCKING1,
        CRASHED0
    } frame_t;

    typedef enum logic[2:0] {
        WAITING,
        RUNNING,
        JUMPING,
        DROPPING,
        DUCKING,
        CRASHED
    } state_t;

    parameter signed DROP_VELOCITY = -5;
    parameter HEIGHT = 47;
    parameter SPEED_DROP_VELOCITY = 1;
    parameter SPEED_DROP_COEFFICIENT = 3;
    parameter START_X_POS = 20;
    parameter WIDTH = 44;
    parameter WIDTH_DUCK = 59;

    parameter GRAVITY = 6;
    parameter SLOW_GRAVITY = 3;
    parameter MAX_JUMP_HEIGHT = 30;
    parameter SLOW_MAX_JUMP_HEIGHT = 50;
    parameter MIN_JUMP_HEIGHT = 30;
    parameter SLOW_MIN_JUMP_HEIGHT = 45;
    parameter signed INITIAL_JUMP_VELOCITY = -10;
    parameter signed SLOW_INITIAL_JUMP_VELOCITY = -20;

    // Position when on the ground.
    parameter GROUND_Y_POS = 150 - HEIGHT - 10;

    // Flash duration in frames.
    parameter FLASH_DURATION = 15;

    // Flash iterations for achievement animation.
    parameter FLASH_ITERATONS = 3;

    import collision_pkg::collision_box_t;

    parameter COLLISION_BOX_COUNT = 6;

    parameter collision_box_t COLLISION_BOX[COLLISION_BOX_COUNT] = '{
        '{22, 0, 17, 16},
        '{1, 18, 30, 9},
        '{10, 35, 14, 8},
        '{1, 24, 29, 5},
        '{5, 30, 21, 4},
        '{9, 34, 15, 4}
    };
    
    parameter collision_box_t COLLISION_BOX_DUCK = '{1, 18, 42, 25};

endpackage

// T-rex game character.
module trex (
    input clk,
    input rst,

    input update,
    input[5:0] timer,

    input[4:0] speed,

    input slow,

    input jump,
    input duck,
    input crack,
    input crash,

    // Immune to collision after a crack.
    output logic immune,

    output logic signed[11:0] x_pos,
    output logic signed[11:0] y_pos,
    output logic[9:0] width,
    output logic[9:0] height,

    output trex_pkg::frame_t frame,
    output logic paint,

    output collision_pkg::collision_box_t
        collision_box[trex_pkg::COLLISION_BOX_COUNT]
);
    import trex_pkg::*;

    import collision_pkg::*;
    import util_func::*;

    state_t state, next_state;

    logic[4:0] gravity;
    assign gravity = slow ? SLOW_GRAVITY : GRAVITY;

    logic[7:0] max_jump_height;
    assign max_jump_height = slow ? SLOW_MAX_JUMP_HEIGHT : MAX_JUMP_HEIGHT;

    logic[7:0] min_jump_height;
    assign min_jump_height = GROUND_Y_POS -
        (slow ? SLOW_MIN_JUMP_HEIGHT : MIN_JUMP_HEIGHT);

    logic signed[9:0] initial_jump_velocity;
    assign initial_jump_velocity = slow ?
        SLOW_INITIAL_JUMP_VELOCITY : INITIAL_JUMP_VELOCITY;

    logic signed[9:0] jump_velocity;
    logic[4:0] gravity_counter;

    logic reached_min_height;

    // Flash when cracked.
    logic[4:0] flash_timer;
    logic[2:0] flash_iterations;

    collision_pkg::collision_box_t
        collision_box_tmp[trex_pkg::COLLISION_BOX_COUNT];

    assign width = state == DUCKING ? WIDTH_DUCK : WIDTH;
    assign height = HEIGHT;

    always_ff @(posedge clk) begin
        if (rst) begin
            state <= WAITING;
            x_pos <= START_X_POS;
            y_pos <= GROUND_Y_POS;
            frame <= WAITING0;
            jump_velocity <= 0;
            gravity_counter <= 0;
            reached_min_height <= 0;
            immune <= 0;
            flash_timer <= 0;
            flash_iterations <= 0;
            paint <= 1;
            for (int i = 0; i < COLLISION_BOX_COUNT; i++) begin
                collision_box[i] <= '{0, 0, 0, 0};
            end
        end else begin
            state <= next_state;

            update_frame();

            collision_box <= collision_box_tmp;

            if (update) begin
                // Cracked, flash t-rex.
                if (crash || state == CRASHED) begin
                    immune <= 0;
                end else if (crack) begin
                    immune <= 1;
                end

                if (immune) begin
                    if (flash_iterations <= FLASH_ITERATONS) begin
                        flash_timer <= flash_timer + 1;
                        if (flash_timer < FLASH_DURATION) begin
                            paint <= 0;
                        end else begin
                            paint <= 1;
                            if (flash_timer > FLASH_DURATION * 2) begin
                                flash_timer <= 0;
                                flash_iterations <= flash_iterations + 1;
                            end
                        end
                    end else begin
                        immune <= 0;
                        flash_timer <= 0;
                        flash_iterations <= 0;
                    end
                end else begin
                    paint <= 1;
                end

                case (next_state)
                    RUNNING: begin
                        // Clear jumping state, e.g. jumping velocity.
                        reset();
                    end
                    JUMPING: begin
                        if (state == WAITING || state == RUNNING) begin
                            start_jump(speed);
                        end else if (state == JUMPING) begin
                            update_jump();
                        end
                    end
                    DROPPING: begin
                        if (state == JUMPING) begin
                            set_speed_drop();
                        end else begin
                            // Speed drop makes Trex fall faster.
                            update_jump(SPEED_DROP_COEFFICIENT);
                        end
                    end
                    CRASHED: begin
                        // Crashed whilst ducking. Trex is standing up so needs
                        // adjustment.
                        if (state == DUCKING) begin
                            x_pos <= x_pos + 1;
                        end
                    end
                endcase
            end
        end
    end

    always_comb begin
        case (state)
            WAITING: begin
                next_state = jump ? RUNNING : WAITING;
            end
            RUNNING: begin
                next_state = jump ? JUMPING : duck ? DUCKING : RUNNING;
            end
            JUMPING: begin
                if (y_pos + jump_velocity > $signed(GROUND_Y_POS)) begin
                    next_state = RUNNING;
                end else begin
                    next_state = duck ? DROPPING : JUMPING;
                end
            end
            DROPPING: begin
                if (y_pos + jump_velocity * SPEED_DROP_COEFFICIENT
                    > GROUND_Y_POS
                ) begin
                    next_state = RUNNING;
                end else begin
                    next_state = duck ? DROPPING : JUMPING;
                end
            end
            DUCKING: begin
                next_state = duck ? DUCKING : RUNNING;
            end
            CRASHED: begin
                next_state = CRASHED;
            end
            default: begin
                next_state = WAITING;
            end
        endcase

        if (crash) begin
            next_state = CRASHED;
        end

        // Preserve state if not received update signal.
        if (!update) begin
            next_state = state;
        end
    end

    always_comb begin
        for (int i = 0; i < COLLISION_BOX_COUNT; i++) begin
            collision_box_tmp[i] = create_adjusted_collision_box(
                state != DUCKING ? COLLISION_BOX[i] : COLLISION_BOX_DUCK,
                x_pos,
                y_pos
            );
        end
    end

    task reset;
        x_pos <= START_X_POS;
        y_pos <= GROUND_Y_POS;
        jump_velocity <= 0;
        gravity_counter <= 0;
    endtask

    // Update frame to render based on the next state.
    task update_frame;
        case (next_state)
            WAITING: begin
                frame <= timer >= 30 ? WAITING0 : WAITING1;
            end
            RUNNING: begin
                frame <= inside_range(timer, 0, 5) ||
                    inside_range(timer, 10, 15) ||
                    inside_range(timer, 20, 25) ||
                    inside_range(timer, 30, 35) ||
                    inside_range(timer, 40, 45) ||
                    inside_range(timer, 50, 55)
                ? RUNNING0 : RUNNING1;
            end
            JUMPING, DROPPING: begin
                frame <= JUMPING0;
            end
            DUCKING: begin
                frame <= inside_range(timer, 0, 10) ||
                    inside_range(timer, 20, 30) ||
                    inside_range(timer, 40, 50)
                ? DUCKING0 : DUCKING1;
            end
            CRASHED: begin
                frame <= CRASHED0;
            end
        endcase
    endtask

    // Initialize a jump.
    task start_jump(logic[3:0] speed);
        // Tweak the jump velocity based on the speed.
        jump_velocity <= initial_jump_velocity - (speed >> 3);
        gravity_counter <= 0;
        reached_min_height <= 0;
    endtask

    // Jump is complete, falling down.
    task end_jump;
        if (reached_min_height && jump_velocity < DROP_VELOCITY) begin
            jump_velocity <= DROP_VELOCITY;
        end
    endtask

    // Set the speed drop. Immediately cancels the current jump.
    task set_speed_drop;
        jump_velocity <= SPEED_DROP_VELOCITY;
    endtask

    // Update frame for a jump.
    task update_jump(int coefficient = 1);
        if (gravity_counter + gravity >= 10) begin
            gravity_counter <= gravity_counter + gravity - 10;
            jump_velocity <= jump_velocity + 1;
            y_pos <= y_pos + (jump_velocity + 1) * coefficient;
        end else begin
            gravity_counter <= gravity_counter + gravity;
            y_pos <= y_pos + jump_velocity * coefficient;
        end

        // Minimum height has been reached.
        if (y_pos < min_jump_height || duck) begin
            reached_min_height = 1;
        end

        // Reached max height.
        if (y_pos < max_jump_height) begin
            end_jump();
        end

        // Jumping signal has been removed.
        if (!jump) begin
            end_jump();
        end
    endtask
    
endmodule
