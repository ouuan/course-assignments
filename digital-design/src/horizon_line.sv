package horizon_line_pkg;
    typedef enum logic[1:0] {
        WAITING,
        RUNNING,
        UPDATING,
        CRASHED
    } state_t;

    parameter WIDTH = 640;
    parameter HEIGHT = 12;
    parameter Y_POS = 127;

    parameter SPEED_SCALE = 1024;
endpackage

// Use two lines to represent the horizon
module horizon_line (
    input clk,
    input rst,

    input update,
    input[14:0] speed,

    input start,
    input crash,

    input bit[10:0] rng_data,

    output logic signed[10:0] x_pos[2],

    output logic bump[2]
);
    import horizon_line_pkg::*;

    state_t state, next_state;

    logic signed[20:0] x_pos_game[2];

    always_comb begin
        next_state = state;
        case (state)
            WAITING: begin
                next_state = RUNNING;
            end
            RUNNING: begin
                if (update) begin
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
            x_pos_game[0] <= 0;
            x_pos_game[1] <= WIDTH * SPEED_SCALE;
            x_pos[0] <= 0;
            x_pos[1] <= WIDTH;
            bump[0] <= 0;
            bump[1] <= 0;
        end else begin
            state <= next_state;
            case (next_state)
                UPDATING: begin
                    run();
                end
            endcase
        end
    end

    // Is this horizon line invisible.
    function logic invisible(logic i);
        return x_pos_game[i] - $signed(speed) <= -$signed(WIDTH) * SPEED_SCALE;
    endfunction

    // Calculate new x pos.
    function logic signed[20:0] get_x_pos(logic i);
        automatic logic signed[20:0] result = x_pos_game[i] - $signed(speed);
        // Move to the other side of the screen.
        if (invisible(i)) begin
            result += $signed(WIDTH) * SPEED_SCALE * 2;
        end
        return result;
    endfunction

    task run;
        for (int i = 0; i < 2; i++) begin
            // Run out of screen. Move to the other side and set new bump type.
            if (invisible(i)) begin
                bump[i] <= rng_data[0];
            end
            x_pos_game[i] <= get_x_pos(i);
            x_pos[i] <= get_x_pos(i) / SPEED_SCALE;
        end
    endtask

endmodule
