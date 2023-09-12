package collision_pkg;
    typedef struct packed {
        logic signed[11:0] x;
        logic signed[11:0] y;
        logic signed[9:0] w;
        logic signed[9:0] h;
    } collision_box_t;

    // Compare two collision boxes for a collision.
    function logic box_compare(
        collision_box_t trex_box,
        collision_box_t obstacle_box
    );
        return (
            trex_box.x < obstacle_box.x + obstacle_box.w &&
            trex_box.x + trex_box.w > obstacle_box.x &&
            trex_box.y < obstacle_box.y + obstacle_box.h &&
            trex_box.y + trex_box.h > obstacle_box.y
        );
    endfunction

    // Adjust the collision box.
    function collision_box_t create_adjusted_collision_box(
        collision_box_t box,
        logic signed[11:0] x,
        logic signed[11:0] y
    );
        return '{box.x + x, box.y + y, box.w, box.h};
    endfunction

endpackage
