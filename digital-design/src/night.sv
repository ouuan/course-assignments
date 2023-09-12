package night_pkg;
    parameter GAME_WIDTH = 640;
    parameter SPEED_SCALE = 1024;

    // Visible when the night rate is above this value.
    parameter VISIBLE_RATE = 32;

    parameter MOON_INIT_X_POS = GAME_WIDTH - 50;
    parameter MOON_Y_POS = 30;
    parameter HEIGHT = 40;
    parameter WIDTH = 20;

    parameter MOON_SPEED = 256;
    parameter STAR_SPEED = 307;

    parameter NUM_STARS = 2;
    parameter SEGMENT_SIZE = GAME_WIDTH / NUM_STARS;
    parameter STAR_SIZE = 9;
    parameter STAR_MAX_Y = 70;

    parameter NUM_PHASES = 7;
endpackage

import night_pkg::NUM_STARS;

module night (
    input clk,
    input rst,

    input update,

    input crash,

    input[9:0] star_x_rand[2],
    input[9:0] star_y_rand[2],

    input[5:0] night_rate,

    output logic signed[10:0] moon_x_pos,
    output logic[9:0] moon_width,
    output logic[2:0] moon_phase,

    output logic signed[10:0] star_x_pos[NUM_STARS],
    output logic[9:0] star_y_pos[NUM_STARS],
    
    output logic activated
);
    import night_pkg::*;

    logic signed[20:0] moon_x_pos_game;
    logic signed[20:0] star_x_pos_game[NUM_STARS];

    assign moon_width = moon_phase == 3 ? WIDTH * 2 : WIDTH;

    function logic activate;
        return night_rate > VISIBLE_RATE;
    endfunction

    always_ff @(posedge clk) begin
        if (rst) begin
            activated <= 0;
            moon_x_pos_game <= MOON_INIT_X_POS * SPEED_SCALE;
            moon_x_pos <= MOON_INIT_X_POS;
            star_x_pos_game <= '{0, 0};
            star_x_pos <= '{0, 0};
            star_y_pos <= '{0, 0};
            moon_phase <= NUM_PHASES - 1;
        end else begin
            if (update) begin
                // Enter the night mode.
                if (!activated && activate) begin
                    activated <= 1;
                    if (moon_phase + 1 == NUM_PHASES) begin
                        moon_phase <= 0;
                    end else begin
                        moon_phase <= moon_phase + 1;
                    end
                    place_stars();
                end else if (!activate) begin
                    activated <= 0;
                end

                if (activated && !crash) begin
                    update_x_pos();
                end
            end
        end
    end

    task place_stars;
        for (int i = 0; i < NUM_STARS; i++) begin
            star_x_pos_game[i] <= (SEGMENT_SIZE * i + star_x_rand[i])
                * SPEED_SCALE;
            star_x_pos[i] <= SEGMENT_SIZE * i + star_x_rand[i];
            star_y_pos[i] <= star_y_rand[i];
        end
    endtask

    function logic signed[20:0] get_x_pos(
        logic signed[20:0] x_pos_game,
        logic[14:0] speed
    );  
        if (x_pos_game - $signed(speed) <
            -$signed(WIDTH) * 2 * SPEED_SCALE
        ) begin
            return GAME_WIDTH * SPEED_SCALE;
        end else begin
            return x_pos_game - $signed(speed);
        end
    endfunction

    task update_x_pos;
        moon_x_pos_game <= get_x_pos(moon_x_pos_game, MOON_SPEED);
        moon_x_pos <= get_x_pos(moon_x_pos_game, MOON_SPEED) / SPEED_SCALE;
        for (int i = 0; i < NUM_STARS; i++) begin
            star_x_pos_game[i] <= get_x_pos(
                star_x_pos_game[i], STAR_SPEED
            );
            star_x_pos[i] <= get_x_pos(
                star_x_pos_game[i], STAR_SPEED
            ) / SPEED_SCALE;
        end
    endtask

endmodule
