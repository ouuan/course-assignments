module palette #(
    MAX_NIGHT_RATE = 63,
    NIGHT_RATE = 0
) (
    input [2:0] palette_index,
    output logic [7:0] red,
    output logic [7:0] green,
    output logic [7:0] blue
);
  task set_colors(input logic [7:0] original_red, input logic [7:0] original_green,
                  input logic [7:0] original_blue);
    red = NIGHT_RATE * 255 / MAX_NIGHT_RATE + (MAX_NIGHT_RATE - NIGHT_RATE * 2) * original_red / MAX_NIGHT_RATE;
    green = NIGHT_RATE * 255 / MAX_NIGHT_RATE + (MAX_NIGHT_RATE - NIGHT_RATE * 2) * original_green / MAX_NIGHT_RATE;
    blue = NIGHT_RATE * 255 / MAX_NIGHT_RATE + (MAX_NIGHT_RATE - NIGHT_RATE * 2) * original_blue / MAX_NIGHT_RATE;
  endtask

  always_comb begin
    case (palette_index)
      3'd1: begin
        set_colors(8'd82, 8'd84, 8'd81);
      end
      3'd2: begin
        set_colors(8'd94, 8'd96, 8'd93);
      end
      3'd3: begin
        set_colors(8'd116, 8'd118, 8'd115);
      end
      3'd4: begin
        set_colors(8'd184, 8'd186, 8'd183);
      end
      3'd5: begin
        set_colors(8'd216, 8'd219, 8'd215);
      end
      3'd6: begin
        set_colors(8'd245, 8'd247, 8'd244);
      end
      3'd7: begin
        set_colors(8'd255, 8'd255, 8'd255);
      end
      default: begin
        red   = NIGHT_RATE * 255 / MAX_NIGHT_RATE;
        green = NIGHT_RATE * 255 / MAX_NIGHT_RATE;
        blue  = NIGHT_RATE * 255 / MAX_NIGHT_RATE;
      end
    endcase
  end
endmodule
