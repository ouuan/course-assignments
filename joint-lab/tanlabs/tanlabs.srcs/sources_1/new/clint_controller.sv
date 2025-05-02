`include "cpu/rv32i_cpu.vh"

module clint_controller #(
    parameter ADDR_WIDTH = 32,
    parameter DATA_WIDTH = 32
) (
    // clk and reset
    input wire clk_i,
    input wire rst_i,

    // wishbone slave interface
    input wire wb_cyc_i,
    input wire wb_stb_i,
    output reg wb_ack_o,
    input wire [ADDR_WIDTH-1:0] wb_adr_i,
    input wire [DATA_WIDTH-1:0] wb_dat_i,
    output reg [DATA_WIDTH-1:0] wb_dat_o,
    input wire [DATA_WIDTH/8-1:0] wb_sel_i,
    input wire wb_we_i
);
    localparam MTIME_ADDR = 8'hBF;
    logic [63:0] mtime;

    /*-- wishbone fsm --*/
    always_ff @(posedge clk_i) begin
        if (rst_i) begin
            wb_ack_o <= 0;
        end else begin
            // every request get ACK-ed immediately
            if (wb_ack_o) begin
                wb_ack_o <= 0;
            end else begin
                wb_ack_o <= wb_stb_i;
            end
        end
    end

    logic [$clog2(SYS_CLOCK_FREQ/1_000_000+1)-1:0] count;

    always_ff @(posedge clk_i) begin
        if (rst_i) begin
            mtime <= 0;
            count <= 0;
        end else begin
            count <= count + 1;
            if (count == SYS_CLOCK_FREQ / 1_000_000) begin
                count <= 0;
                mtime <= mtime + 1;
            end
        end
    end

    // read logic
    always_ff @(posedge clk_i) begin
        if (rst_i) begin

        end else if (wb_stb_i && !wb_we_i) begin
            case (wb_adr_i[15:8])
                MTIME_ADDR: begin
                    case (wb_adr_i[2])
                        1'b0: begin
                            if (wb_sel_i[0]) wb_dat_o[ 7: 0] <= mtime[ 7: 0];
                            if (wb_sel_i[1]) wb_dat_o[15: 8] <= mtime[15: 8];
                            if (wb_sel_i[2]) wb_dat_o[23:16] <= mtime[23:16];
                            if (wb_sel_i[3]) wb_dat_o[31:24] <= mtime[31:24];
                        end
                        1'b1: begin
                            if (wb_sel_i[0]) wb_dat_o[ 7: 0] <= mtime[39:32];
                            if (wb_sel_i[1]) wb_dat_o[15: 8] <= mtime[47:40];
                            if (wb_sel_i[2]) wb_dat_o[23:16] <= mtime[55:48];
                            if (wb_sel_i[3]) wb_dat_o[31:24] <= mtime[63:56];
                        end
                        default: ;
                    endcase
                end
                default: ;  // do nothing
            endcase
        end
    end

endmodule
