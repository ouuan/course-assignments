module spike_filter (
    input clk,
    input rst,
    input in,
    output logic out
);
  parameter WIDTH = 2;
  parameter logic INITIAL_VALUE = 1'b1;

  logic [WIDTH-1:0] cnt = 0;

  always_ff @(posedge clk) begin
    if (rst) begin
      for (integer i = 0; i < WIDTH; ++i) cnt[i] <= INITIAL_VALUE;
      out <= INITIAL_VALUE;
    end else begin
      if (cnt == 0) out <= 0;
      else if (~in) cnt <= cnt - 1;
      if (&cnt) out <= 1;
      else if (in) cnt <= cnt + 1;
    end
  end
endmodule
