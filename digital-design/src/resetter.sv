module resetter (
    input clk,
    input reset_btn,
    output logic rst_out
);
  localparam WIDTH = 16;
  localparam CNT = 2 ** WIDTH - 1;

  logic [WIDTH-1:0] cnt = 0;
  logic last_reset_btn;

  always_ff @(posedge clk) begin
    last_reset_btn <= reset_btn;
    if (cnt != CNT) begin
      cnt <= cnt + 1;
      rst_out <= 1;
    end else begin
      rst_out <= last_reset_btn;
    end
  end
endmodule
