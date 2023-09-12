package cloud_pkg;
    typedef enum {
        WAITING,
        RUNNING,
        UPDATING,
        CRASHED
    } state_t;

    parameter WIDTH = 46;
    parameter HEIGHT = 14;

    parameter MIN_SKY_LEVEL = 71;
    parameter MAX_SKY_LEVEL = 30;

    parameter MIN_CLOUD_GAP = 100;
    parameter MAX_CLOUD_GAP = 400;

    parameter SPEED_COEFFICIENT = 205;

    parameter GAME_WIDTH = 640;
    parameter SPEED_SCALE = 1024;

endpackage

// Cloud background item.
// Similar to an obstacle object but without collision boxes.
module cloud (
    input clk,
    input rst,

    input update,
    input[14:0] speed,

    input start,
    input crash,

    input[9:0] level_rand,
    input[10:0] gap_rand, 

    output logic remove,
    output logic[10:0] gap,
    output logic visible,

    output logic signed[10:0] x_pos,
    output logic[9:0] y_pos
);  
    import cloud_pkg::*;

    state_t state, next_state;

    logic signed[20:0] x_pos_game;

    assign visible = x_pos + $signed(WIDTH) > 0;

    always_comb begin
        next_state = state;
        case (state)
            WAITING: begin
                if (update && start) begin
                    next_state = RUNNING;
                end
            end
            RUNNING: begin
                if (remove) begin
                    next_state = WAITING;
                end else if (update) begin
                    next_state = UPDATING;
                end
            end
            UPDATING: begin
                next_state = RUNNING;
            end
            CRASHED: begin
                next_state = CRASHED;
            end
        endcase
        if (crash) begin
            next_state = CRASHED;
        end
    end

    always_ff @(posedge clk) begin
        if (rst) begin
            state <= WAITING;
            x_pos_game <= 0;
            x_pos <= 0;
            y_pos <= 0;

            remove <= 0;
            gap <= 0;
        end else begin
            state <= next_state;

            case (next_state)
                RUNNING: begin
                    if (state == WAITING) begin
                        init();
                    end
                end
                UPDATING: begin
                    run();
                end
            endcase
        end
    end

    task init;
        remove <= 0;
        x_pos_game <= GAME_WIDTH * SPEED_SCALE;
        x_pos <= GAME_WIDTH;
        y_pos <= level_rand + MAX_SKY_LEVEL;
        gap <= gap_rand + MIN_CLOUD_GAP;
    endtask

    function logic signed[20:0] get_x_pos;
        return x_pos_game - $signed(speed / SPEED_SCALE * SPEED_COEFFICIENT);
    endfunction

    task run;
        x_pos_game <= get_x_pos();
        x_pos <= get_x_pos() / SPEED_SCALE;

        if (!visible) begin
            remove <= 1;
        end
    endtask

endmodule
