import runner_pkg::sprite_t;
import runner_pkg::pos_t;
import runner_pkg::RENDER_SLOTS;

module runner_sim;

    logic clk = 0, rst = 0, update, jumping = 0, ducking = 0;

    sprite_t sprite[RENDER_SLOTS];
    pos_t pos[RENDER_SLOTS];

    parameter FPS = 60;

    runner dut (
        .clk,
        .rst,
        .jumping,
        .ducking,

        .sprite,
        .pos
    );

    int counter = 0;

    assign update = counter % 7 == 0;

    initial begin
        forever begin
            #5 clk = 0;
            counter++;
            #5 clk = 1;
        end
    end

    initial begin
        #100 rst = 1;
        #100 rst = 0;
        #100 jumping = 1;
        #1000 jumping = 0;

        #20000 jumping = 1;
        #1000 jumping = 0;
        
        #200000 $finish;
    end

endmodule;