// Handles displaying the distance meter.
package distance_meter_pkg;
    typedef logic[3:0] distance_t[MAX_DISTANCE_UNITS];

    parameter GAME_WIDTH = 640;
    parameter SPEED_SCALE = 1024;

    parameter WIDTH = 10;
    parameter HEIGHT = 13;
    parameter DEST_WIDTH = 11;

    // Distance meter config.

    // Number of digits.
    parameter MAX_DISTANCE_UNITS = 5;
    
    // "HI" chars in high score.
    parameter HIGH_SCORE_OFFSET = 3;
    parameter MAX_HIGH_SCORE_UNITS = MAX_DISTANCE_UNITS + HIGH_SCORE_OFFSET;

    // Distance that causes achievement animation.
    parameter ACHIEVEMENT_DISTANCE = 100;

    // Used for conversion from pixel distance to a scaled unit.
    parameter COEFFICIENT = 40 * SPEED_SCALE;

    // Flash duration in frames.
    parameter FLASH_DURATION = 15;

    // Flash iterations for achievement animation.
    parameter FLASH_ITERATONS = 3;

    parameter INVERT_DISTANCE = 700;
    parameter SLOW_INVERT_DISTANCE = 350;

    parameter X = (GAME_WIDTH - (DEST_WIDTH * (MAX_DISTANCE_UNITS + 1))) * 2;
    parameter Y = 20;

    parameter HIGH_SCORE_X = X - MAX_DISTANCE_UNITS * 2 * WIDTH * 2;

endpackage

import distance_meter_pkg::*;

// Distance meter module.
module distance_meter(
    input clk,
    input rst,

    input update,
    input restart,

    input[14:0] speed,

    input slow,

    output distance_t digits,
    output logic[3:0] high_score[MAX_HIGH_SCORE_UNITS],
    output logic paint,

    output logic invert_trigger
);

    logic[15:0] distance_counter;
    distance_t distance;

    logic achievement;
    logic[6:0] achievement_counter;
    distance_t achievement_distance;
    logic[4:0] flash_timer;
    logic[2:0] flash_iterations;

    logic[16:0] distance_val;
    logic[16:0] high_score_val;

    logic[9:0] invert_counter;

    assign digits = achievement ? achievement_distance : distance;

    // The "HI" characters.
    assign high_score[0] = 10;
    assign high_score[1] = 11;
    assign high_score[2] = 12;

    always_ff @(posedge clk) begin
        if (rst || restart) begin
            paint <= 1;
            distance_counter <= 0;
            achievement <= 0;
            achievement_counter <= 0;
            flash_timer <= 0;
            flash_iterations <= 0;
            for (int i = 0; i < MAX_DISTANCE_UNITS; i++) begin
                distance[i] <= 0;
                achievement_distance[i] <= 0;
            end
            distance_val <= 0;
            invert_counter <= 0;
            invert_trigger <= 0;

            if (rst) begin
                high_score_val <= 0;
                for (int i = 0; i < MAX_DISTANCE_UNITS; i++) begin
                    high_score[HIGH_SCORE_OFFSET + i] <= 0;
                end
            end
        end else if (!speed) begin
            // Compare and set high score
            if (distance_val > high_score_val) begin
                high_score_val <= distance_val;
                for (int i = 0; i < MAX_DISTANCE_UNITS; i++) begin
                    high_score[HIGH_SCORE_OFFSET + i] <= distance[i];
                end
            end

            achievement <= 0;
            paint <= 1;
        end else if (update) begin
            if (distance_counter + speed < COEFFICIENT) begin
                distance_counter <= distance_counter + speed;
            end else begin
                distance_counter <= distance_counter + speed - COEFFICIENT;

                distance_val <= distance_val + 1;

                // Trigger night mode.
                if (invert_counter + 1 ==
                    (slow ? SLOW_INVERT_DISTANCE : INVERT_DISTANCE)
                ) begin
                    invert_counter <= 0;
                    invert_trigger <= 1;
                end else begin
                    invert_counter <= invert_counter + 1;
                    invert_trigger <= 0;
                end

                for (int i = 0; i < MAX_DISTANCE_UNITS; i++) begin
                    // Didn't reach max distance, increment.
                    if (distance[i] != 9) begin
                        distance <= incr();

                        if (achievement_counter == ACHIEVEMENT_DISTANCE) begin
                            achievement_counter <= 1;
                        end else begin
                            achievement_counter <= achievement_counter + 1;
                        end

                        break;
                    end
                end
            end

            // Achievement unlocked.
            if (achievement_counter == ACHIEVEMENT_DISTANCE) begin
                achievement <= 1;
                achievement_distance <= distance;
                flash_iterations <= 0;
            end

            if (achievement) begin
                // Flash score.
                if (flash_iterations <= FLASH_ITERATONS) begin
                    flash_timer <= flash_timer + update;

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
                    achievement <= 0;
                    flash_timer <= 0;
                    flash_iterations <= 0;
                end
            end
        end
    end

    // Increments the distance digits.
    function distance_t incr;
        automatic distance_t digits = distance;

        for (int i = MAX_DISTANCE_UNITS - 1; i >= 0; i--) begin
            if (digits[i] == 9) begin
                digits[i] = 0;
            end else begin
                digits[i] = digits[i] + 1;
                break;
            end
        end

        return digits;
    endfunction

endmodule
