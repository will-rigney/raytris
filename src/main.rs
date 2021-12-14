use rand::prelude::*;
use raylib::prelude::*;

// cheat and copy the rosetta code go implementation
const SQUARE_SIZE: i32 = 20;
const GRID_HORIZONTAL_SIZE: usize = 12;
const GRID_VERTICAL_SIZE: usize = 20;
const LATERAL_SPEED: u16 = 10;
const TURNING_SPEED: u16 = 12;
const FAST_FALL_AWAIT_COUNTER: u16 = 30;
const FADING_TIME: u16 = 33;

// these maybe should have associated values or smth
#[derive(Clone, Copy, PartialEq, Eq)]
enum GridSquare {
    Empty,  // nothing in the grid square
    Moving, // square of piece in flight (aka has colour)
    Full,   // square is full, no longer in flight
    Block,  // dunno
    Fading,
}

// next defines bunch of variables
// should be in a mutable struct
struct Game {
    screen_width: i32,
    screen_height: i32,
    game_over: bool,
    pause: bool,

    // These variables keep track of the active piece position
    piece_position_x: usize,
    piece_position_y: usize,

    // These variables record the active and incoming piece colors
    piece_color: Color,
    incoming_piece_color: Color,

    // Statistics
    level: u16,
    lines: u16,

    // Based on level
    gravity_speed: u16,

    // grid
    grid: [[GridSquare; GRID_VERTICAL_SIZE]; GRID_HORIZONTAL_SIZE],
    piece: [[GridSquare; 4]; 4],
    incoming_piece: [[GridSquare; 4]; 4],

    // game parameters
    fading_colour: Color,
    begin_play: bool,
    piece_active: bool,
    detection: bool,
    line_to_delete: bool,

    // counters
    gravity_movement_counter: u16,
    lateral_movement_counter: u16,
    turn_movement_counter: u16,
    fast_fall_movement_counter: u16,
    fade_line_counter: u16,
}

impl Game {
    fn new() -> Self {
        let mut grid = [[GridSquare::Empty; GRID_VERTICAL_SIZE]; GRID_HORIZONTAL_SIZE];
        for i in 0..GRID_HORIZONTAL_SIZE {
            for j in 0..GRID_VERTICAL_SIZE {
                if (j == GRID_VERTICAL_SIZE - 1) || (i == 0) || (i == GRID_HORIZONTAL_SIZE - 1) {
                    grid[i][j] = GridSquare::Block;
                }
            }
        }

        Game {
            screen_width: 600,
            screen_height: 450,
            game_over: false,
            pause: false,

            // These variables keep track of the active piece position
            piece_position_x: 0,
            piece_position_y: 0,

            // These variables record the active and incoming piece colors
            piece_color: Color::GRAY,
            incoming_piece_color: Color::GRAY,

            // Statistics
            level: 1,
            lines: 0,

            // Based on level
            gravity_speed: 30,
            grid,
            piece: [[GridSquare::Empty; 4]; 4],
            incoming_piece: [[GridSquare::Empty; 4]; 4],
            fading_colour: Color::GRAY,
            begin_play: true,
            piece_active: false,
            detection: false,
            line_to_delete: false,
            gravity_movement_counter: 0,
            lateral_movement_counter: 0,
            turn_movement_counter: 0,
            fast_fall_movement_counter: 0,
            fade_line_counter: 0,
        }
    }

    fn update(&mut self, rl: &RaylibHandle) {
        // do nothing if the game is over
        if self.game_over {
            if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                // reinit the game
                *self = Game::new();
            }
            return;
        }
        if !self.pause {
            if !self.line_to_delete {
                if !self.piece_active {
                    // get another piece
                    self.piece_active = self.create_piece();
                    // we leave a little time before starting the fast falling down
                    self.fast_fall_movement_counter = 0;
                } else {
                    // counters update
                    self.fast_fall_movement_counter += 1;
                    self.gravity_movement_counter += 1;
                    self.lateral_movement_counter += 1;
                    self.turn_movement_counter += 1;

                    // make sure to move if we've pressed the key this frame
                    if rl.is_key_pressed(KeyboardKey::KEY_LEFT)
                        || rl.is_key_pressed(KeyboardKey::KEY_RIGHT)
                    {
                        self.lateral_movement_counter = LATERAL_SPEED;
                    }
                    if rl.is_key_pressed(KeyboardKey::KEY_UP) {
                        self.turn_movement_counter = TURNING_SPEED;
                    }

                    // fall down
                    if rl.is_key_pressed(KeyboardKey::KEY_DOWN)
                        && self.fast_fall_movement_counter >= FAST_FALL_AWAIT_COUNTER
                    {
                        // make sure piece will fall this frame
                        self.gravity_movement_counter += self.gravity_speed;
                    }

                    if self.gravity_movement_counter >= self.gravity_speed {
                        // basic falling movement
                        self.check_detection();

                        // check if piece has collided with another piece
                        // or with the boundaries
                        self.resolve_falling_movement();

                        // check if we completed a line and if so erase the line
                        // and pull down lines above
                        self.check_completion();

                        self.gravity_movement_counter = 0;
                    }

                    // move laterally at player's will (??)
                    if self.lateral_movement_counter >= LATERAL_SPEED {
                        // update the lateral movement and, if successful,
                        // reset the later counter
                        if !self.resolve_lateral_movement(rl) {
                            self.lateral_movement_counter = 0
                        }
                    }

                    // turn the piece at the players will
                    if self.turn_movement_counter >= TURNING_SPEED {
                        // update the turning movement and reset turning counter
                        if self.resolve_turn_movement(rl) {
                            self.turn_movement_counter = 0;
                        }
                    }
                }

                // game over logic
                for j in 0..2 {
                    for i in 1..GRID_HORIZONTAL_SIZE {
                        if self.grid[i][j] == GridSquare::Full {
                            self.game_over = true;
                        }
                    }
                }
            } else {
                // animation when deleting lines
                self.fade_line_counter += 1;

                // todo: magic numbers
                if self.fade_line_counter % 8 < 4 {
                    self.fading_colour = Color::MAROON;
                } else {
                    self.fading_colour = Color::GRAY;
                }

                if self.fade_line_counter >= FADING_TIME {
                    self.delete_complete_lines();
                    self.fade_line_counter = 0;
                    self.line_to_delete = false;
                    self.lines += 1;
                }
            }
        }
    }

    fn draw(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let mut d = rl.begin_drawing(thread);

        d.clear_background(Color::WHITE);

        if !self.game_over {
            // draw gameplay area
            // todo: should have an int vector (just struct w two fields, don't need math)
            let mut offset = Vector2 {
                x: self.screen_width as f32 / 2.0
                    - GRID_HORIZONTAL_SIZE as f32 * SQUARE_SIZE as f32
                    - 50.0, // todo: very magic numbers
                y: self.screen_height as f32 / 2.0
                    - (GRID_VERTICAL_SIZE - 1) as f32 * SQUARE_SIZE as f32 / 2.0
                    + SQUARE_SIZE as f32 * 2.0
                    - 50.0,
            };

            let controller = offset.x;

            for j in 0..GRID_VERTICAL_SIZE {
                for i in 0..GRID_HORIZONTAL_SIZE {
                    // draw each square of the grid
                    // ox, oy := int32(offset.X), int32(offset.Y)
                    let ox = offset.x as i32;
                    let oy = offset.y as i32;
                    if self.grid[i][j] == GridSquare::Empty {
                        d.draw_line(ox, oy, ox + SQUARE_SIZE, oy, Color::LIGHTGRAY);
                        d.draw_line(ox, oy, ox, oy + SQUARE_SIZE, Color::LIGHTGRAY);
                        d.draw_line(
                            ox + SQUARE_SIZE,
                            oy,
                            ox + SQUARE_SIZE,
                            oy + SQUARE_SIZE,
                            Color::LIGHTGRAY,
                        );
                        d.draw_line(
                            ox,
                            oy + SQUARE_SIZE,
                            ox + SQUARE_SIZE,
                            oy + SQUARE_SIZE,
                            Color::LIGHTGRAY,
                        );
                        offset.x += SQUARE_SIZE as f32;
                    } else if self.grid[i][j] == GridSquare::Full {
                        d.draw_rectangle(ox, oy, SQUARE_SIZE, SQUARE_SIZE, Color::GRAY);
                        offset.x += SQUARE_SIZE as f32;
                    } else if self.grid[i][j] == GridSquare::Moving {
                        d.draw_rectangle(ox, oy, SQUARE_SIZE, SQUARE_SIZE, self.piece_color);
                        offset.x += SQUARE_SIZE as f32;
                    } else if self.grid[i][j] == GridSquare::Block {
                        d.draw_rectangle(ox, oy, SQUARE_SIZE, SQUARE_SIZE, Color::LIGHTGRAY);
                        offset.x += SQUARE_SIZE as f32;
                    } else if self.grid[i][j] == GridSquare::Fading {
                        d.draw_rectangle(ox, oy, SQUARE_SIZE, SQUARE_SIZE, self.fading_colour);
                        offset.x += SQUARE_SIZE as f32;
                    }
                }
                offset.x = controller;
                offset.y += SQUARE_SIZE as f32;
            }

            // draw incoming piece (hard-coded)
            offset.x = 500_f32;
            offset.y = 45_f32;

            let controller = offset.x;

            for j in 0..4 {
                for i in 0..4 {
                    let ox = offset.x as i32;
                    let oy = offset.y as i32;

                    if self.incoming_piece[i][j] == GridSquare::Empty {
                        d.draw_line(ox, oy, ox + SQUARE_SIZE, oy, Color::LIGHTGRAY);
                        d.draw_line(ox, oy, ox, oy + SQUARE_SIZE, Color::LIGHTGRAY);
                        d.draw_line(
                            ox + SQUARE_SIZE,
                            oy,
                            ox + SQUARE_SIZE,
                            oy + SQUARE_SIZE,
                            Color::LIGHTGRAY,
                        );
                        d.draw_line(
                            ox,
                            oy + SQUARE_SIZE,
                            ox + SQUARE_SIZE,
                            oy + SQUARE_SIZE,
                            Color::LIGHTGRAY,
                        );
                        offset.x += SQUARE_SIZE as f32;
                    } else if self.incoming_piece[i][j] == GridSquare::Moving {
                        d.draw_rectangle(
                            ox,
                            oy,
                            SQUARE_SIZE,
                            SQUARE_SIZE,
                            self.incoming_piece_color,
                        );
                        offset.x += SQUARE_SIZE as f32;
                    }
                }
                offset.x = controller;
                offset.y += SQUARE_SIZE as f32;
            }

            let ox = offset.x as i32;
            let oy = offset.y as i32;

            // text
            d.draw_text("INCOMING:", ox, oy - 100, 10, Color::GRAY);
            d.draw_text(
                &format!("LINES:     {}", self.lines),
                ox,
                oy + 20,
                10,
                Color::GRAY,
            );

            if self.pause {
                d.draw_text(
                    "GAME PAUSED",
                    self.screen_width / 2,
                    self.screen_height / 2,
                    40,
                    Color::GRAY,
                );
            }
        } else {
            d.draw_text(
                "PRESS [ENTER] TO PLAY AGAIN",
                d.get_screen_width() / 2,
                d.get_screen_height() / 2,
                20,
                Color::GRAY,
            );
        }
    }

    /// move the incoming piece into play & get a new incoming piece
    /// this function works
    fn create_piece(&mut self) -> bool {
        // new piece position at centre of top of board
        self.piece_position_x = (GRID_HORIZONTAL_SIZE - 4) / 2; // todo: magic numbers
        self.piece_position_y = 0;

        // if the game is starting and we are creating the first piece,
        // we create an extra one
        if self.begin_play {
            self.get_random_piece();
            self.begin_play = false;
        }

        // assign the incoming piece to the actual piece
        for i in 0..4 {
            for j in 0..4 {
                self.piece[i][j] = self.incoming_piece[i][j];
            }
        }
        self.piece_color = self.incoming_piece_color;

        // assign a new random piece to the incoming piece
        self.get_random_piece();

        // assign the piece to the grid
        for i in 0..4 {
            for j in 0..4 {
                if self.piece[i][j] == GridSquare::Moving {
                    self.grid[i + self.piece_position_x][j] = GridSquare::Moving;
                }
            }
        }
        // todo: no point to this return
        true
    }

    /// get a random piece and assign to the self.piece + self.incoming_piece
    /// this function actually works
    /// todo: change to not mutate internal state but allow assignment outside
    fn get_random_piece(&mut self) {
        let mut rng = thread_rng();
        let random = rng.gen_range(0..7);

        for i in 0..4 {
            for j in 0..4 {
                self.incoming_piece[i][j] = GridSquare::Empty;
            }
        }

        match random {
            // square
            0 => {
                self.incoming_piece[1][1] = GridSquare::Moving;
                self.incoming_piece[2][1] = GridSquare::Moving;
                self.incoming_piece[1][2] = GridSquare::Moving;
                self.incoming_piece[2][2] = GridSquare::Moving;
                self.incoming_piece_color = Color::YELLOW;
            }
            // L
            1 => {
                self.incoming_piece[1][0] = GridSquare::Moving;
                self.incoming_piece[1][1] = GridSquare::Moving;
                self.incoming_piece[1][2] = GridSquare::Moving;
                self.incoming_piece[2][2] = GridSquare::Moving;
                self.incoming_piece_color = Color::BLUE;
            }
            // J (inverted L)
            2 => {
                self.incoming_piece[1][2] = GridSquare::Moving;
                self.incoming_piece[2][0] = GridSquare::Moving;
                self.incoming_piece[2][1] = GridSquare::Moving;
                self.incoming_piece[2][2] = GridSquare::Moving;
                self.incoming_piece_color = Color::BROWN;
            }
            // I (straight)
            3 => {
                self.incoming_piece[0][1] = GridSquare::Moving;
                self.incoming_piece[1][1] = GridSquare::Moving;
                self.incoming_piece[2][1] = GridSquare::Moving;
                self.incoming_piece[3][1] = GridSquare::Moving;
                self.incoming_piece_color = Color::SKYBLUE;
            }
            // T (cross cut)
            4 => {
                self.incoming_piece[1][0] = GridSquare::Moving;
                self.incoming_piece[1][1] = GridSquare::Moving;
                self.incoming_piece[1][2] = GridSquare::Moving;
                self.incoming_piece[2][1] = GridSquare::Moving;
                self.incoming_piece_color = Color::PURPLE;
            }
            // S
            5 => {
                self.incoming_piece[1][1] = GridSquare::Moving;
                self.incoming_piece[2][1] = GridSquare::Moving;
                self.incoming_piece[2][2] = GridSquare::Moving;
                self.incoming_piece[3][2] = GridSquare::Moving;
                self.incoming_piece_color = Color::GREEN;
            }
            // Z (inverted S)
            6 => {
                self.incoming_piece[1][2] = GridSquare::Moving;
                self.incoming_piece[2][2] = GridSquare::Moving;
                self.incoming_piece[2][1] = GridSquare::Moving;
                self.incoming_piece[3][1] = GridSquare::Moving;
                self.incoming_piece_color = Color::RED;
            }

            _ => panic!("generated random number outside range!"),
        }
    }

    fn resolve_falling_movement(&mut self) {
        // if we've finished moving this piece, we stop it
        if self.detection {
            for j in (0..=GRID_VERTICAL_SIZE - 2).rev() {
                for i in 1..GRID_HORIZONTAL_SIZE {
                    if self.grid[i][j] == GridSquare::Moving {
                        self.grid[i][j] = GridSquare::Full;
                        self.detection = false;
                        self.piece_active = false;
                    }
                }
            }
        }
        // we move down the piece
        else {
            for j in (0..=GRID_VERTICAL_SIZE - 2).rev() {
                for i in 1..GRID_HORIZONTAL_SIZE - 1 {
                    if self.grid[i][j] == GridSquare::Moving {
                        self.grid[i][j + 1] = GridSquare::Moving;
                        self.grid[i][j] = GridSquare::Empty;
                    }
                }
            }

            self.piece_position_y += 1;
        }
    }

    fn resolve_lateral_movement(&mut self, rl: &RaylibHandle) -> bool {
        let mut collision = false;

        // piece movement
        // move left
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            // check if it's possible to move left
            for j in (0..=(GRID_VERTICAL_SIZE - 2)).rev() {
                for i in 1..GRID_HORIZONTAL_SIZE - 1 {
                    if self.grid[i][j] == GridSquare::Moving
                        && (i - 1 == 0 || self.grid[i - 1][j] == GridSquare::Full)
                    {
                        collision = true;
                    }
                }
            }

            // if able, move left
            if !collision {
                for j in (0..=GRID_VERTICAL_SIZE - 2).rev() {
                    // check the matrix from left to right
                    for i in 1..GRID_HORIZONTAL_SIZE - 1 {
                        if self.grid[i][j] == GridSquare::Moving {
                            self.grid[i - 1][j] = GridSquare::Moving;
                            self.grid[i][j] = GridSquare::Empty;
                        }
                    }
                }

                self.piece_position_x -= 1;
            }
        }
        // move right
        else if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            for j in (0..=GRID_VERTICAL_SIZE - 2).rev() {
                for i in 1..GRID_HORIZONTAL_SIZE - 1 {
                    if self.grid[i][j] == GridSquare::Moving
                        && (i + 1 == GRID_HORIZONTAL_SIZE - 1
                            || self.grid[i + 1][j] == GridSquare::Full)
                    {
                        collision = true;
                    }
                }
            }

            // if able, move right
            if !collision {
                for j in (0..=GRID_VERTICAL_SIZE - 2).rev() {
                    // check matrix from right to left
                    for i in (1..=GRID_HORIZONTAL_SIZE - 1).rev() {
                        // move everything to the right
                        if self.grid[i][j] == GridSquare::Moving {
                            self.grid[i + 1][j] = GridSquare::Moving;
                            self.grid[i][j] = GridSquare::Empty;
                        }
                    }
                }

                self.piece_position_x += 1;
            }
        }

        collision
    }

    // bug in here that overwrites block grid pieces with empty
    // is there some way to make a hook if this happens?
    fn resolve_turn_movement(&mut self, rl: &RaylibHandle) -> bool {
        // input for turning the piece
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            let mut aux: GridSquare;

            let mut skip = false;

            // check all turning possibilities
            // have to add extra bounds checks not needed in go
            // todo: rewrite to be more rust / human friendly
            if self.piece_position_x + 3 < GRID_HORIZONTAL_SIZE
                && self.grid[self.piece_position_x + 3][self.piece_position_y] == GridSquare::Moving
                && self.grid[self.piece_position_x][self.piece_position_y] != GridSquare::Empty
                && self.grid[self.piece_position_x][self.piece_position_y] != GridSquare::Moving
            {
                skip = true;
            }
            if self.piece_position_x + 3 < GRID_HORIZONTAL_SIZE
                && self.piece_position_y + 3 < GRID_VERTICAL_SIZE
                && self.grid[self.piece_position_x + 3][self.piece_position_y + 3]
                    == GridSquare::Moving
                && self.grid[self.piece_position_x + 3][self.piece_position_y] != GridSquare::Empty
                && self.grid[self.piece_position_x + 3][self.piece_position_y] != GridSquare::Moving
            {
                skip = true;
            }
            if self.piece_position_x + 3 < GRID_HORIZONTAL_SIZE
                && self.piece_position_y + 3 < GRID_VERTICAL_SIZE
                && self.grid[self.piece_position_x][self.piece_position_y + 3] == GridSquare::Moving
                && self.grid[self.piece_position_x + 3][self.piece_position_y + 3]
                    != GridSquare::Empty
                && self.grid[self.piece_position_x + 3][self.piece_position_y + 3]
                    != GridSquare::Moving
            {
                skip = true;
            }
            if self.piece_position_y + 3 < GRID_VERTICAL_SIZE
                && self.grid[self.piece_position_x][self.piece_position_y] == GridSquare::Moving
                && self.grid[self.piece_position_x][self.piece_position_y + 3] != GridSquare::Empty
                && self.grid[self.piece_position_x][self.piece_position_y + 3] != GridSquare::Moving
            {
                skip = true;
            }
            if self.piece_position_x + 1 < GRID_HORIZONTAL_SIZE
                && self.piece_position_y + 2 < GRID_VERTICAL_SIZE
                && self.grid[self.piece_position_x + 1][self.piece_position_y] == GridSquare::Moving
                && self.grid[self.piece_position_x][self.piece_position_y + 2] != GridSquare::Empty
                && self.grid[self.piece_position_x][self.piece_position_y + 2] != GridSquare::Moving
            {
                skip = true;
            }
            // 6
            if self.piece_position_x + 3 < GRID_HORIZONTAL_SIZE
                && self.piece_position_y + 1 < GRID_VERTICAL_SIZE
                && self.grid[self.piece_position_x + 3][self.piece_position_y + 1]
                    == GridSquare::Moving
                && self.grid[self.piece_position_x + 1][self.piece_position_y] != GridSquare::Empty
                && self.grid[self.piece_position_x + 1][self.piece_position_y] != GridSquare::Moving
            {
                skip = true;
            }
            if self.piece_position_x + 2 < GRID_HORIZONTAL_SIZE
                && self.piece_position_y + 3 < GRID_VERTICAL_SIZE
                && self.grid[self.piece_position_x + 2][self.piece_position_y + 3]
                    == GridSquare::Moving
                && self.grid[self.piece_position_x + 3][self.piece_position_y + 1]
                    != GridSquare::Empty
                && self.grid[self.piece_position_x + 3][self.piece_position_y + 1]
                    != GridSquare::Moving
            {
                skip = true;
            }
            if self.piece_position_x + 2 < GRID_HORIZONTAL_SIZE
                && self.piece_position_y + 1 < GRID_VERTICAL_SIZE
                && self.grid[self.piece_position_x][self.piece_position_y] == GridSquare::Moving
                && self.grid[self.piece_position_x + 2][self.piece_position_y + 1]
                    != GridSquare::Empty
                && self.grid[self.piece_position_x + 2][self.piece_position_y + 1]
                    != GridSquare::Moving
            {
                skip = true;
            }
            if self.piece_position_x + 2 < GRID_HORIZONTAL_SIZE
                && self.piece_position_y + 1 < GRID_VERTICAL_SIZE
                && self.grid[self.piece_position_x + 2][self.piece_position_y] == GridSquare::Moving
                && self.grid[self.piece_position_x][self.piece_position_y + 1] != GridSquare::Empty
                && self.grid[self.piece_position_x][self.piece_position_y + 1] != GridSquare::Moving
            {
                skip = true;
            }
            if self.piece_position_x + 3 < GRID_HORIZONTAL_SIZE
                && self.piece_position_y + 2 < GRID_VERTICAL_SIZE
                && self.grid[self.piece_position_x + 3][self.piece_position_y + 2]
                    == GridSquare::Moving
                && self.grid[self.piece_position_x + 2][self.piece_position_y] != GridSquare::Empty
                && self.grid[self.piece_position_x + 2][self.piece_position_y] != GridSquare::Moving
            {
                skip = true;
            }
            if self.piece_position_x + 3 < GRID_HORIZONTAL_SIZE
                && self.piece_position_y + 2 < GRID_VERTICAL_SIZE
                && self.grid[self.piece_position_x + 1][self.piece_position_y + 3]
                    == GridSquare::Moving
                && self.grid[self.piece_position_x + 3][self.piece_position_y + 2]
                    != GridSquare::Empty
                && self.grid[self.piece_position_x + 3][self.piece_position_y + 2]
                    != GridSquare::Moving
            {
                skip = true;
            }
            if self.piece_position_x + 1 < GRID_HORIZONTAL_SIZE
                && self.piece_position_y + 3 < GRID_VERTICAL_SIZE
                && self.grid[self.piece_position_x][self.piece_position_y + 1] == GridSquare::Moving
                && self.grid[self.piece_position_x + 1][self.piece_position_y + 3]
                    != GridSquare::Empty
                && self.grid[self.piece_position_x + 1][self.piece_position_y + 3]
                    != GridSquare::Moving
            {
                skip = true;
            }
            if self.piece_position_x + 1 < GRID_HORIZONTAL_SIZE
                && self.piece_position_y + 2 < GRID_VERTICAL_SIZE
                && self.grid[self.piece_position_x + 1][self.piece_position_y + 1]
                    == GridSquare::Moving
                && self.grid[self.piece_position_x + 1][self.piece_position_y + 2]
                    != GridSquare::Empty
                && self.grid[self.piece_position_x + 1][self.piece_position_y + 2]
                    != GridSquare::Moving
            {
                skip = true;
            }
            if self.piece_position_x + 2 < GRID_HORIZONTAL_SIZE
                && self.piece_position_y + 1 < GRID_VERTICAL_SIZE
                && self.grid[self.piece_position_x + 2][self.piece_position_y + 1]
                    == GridSquare::Moving
                && self.grid[self.piece_position_x + 1][self.piece_position_y + 1]
                    != GridSquare::Empty
                && self.grid[self.piece_position_x + 1][self.piece_position_y + 1]
                    != GridSquare::Moving
            {
                skip = true;
            }
            if self.piece_position_x + 2 < GRID_HORIZONTAL_SIZE
                && self.piece_position_y + 2 < GRID_VERTICAL_SIZE
                && self.grid[self.piece_position_x + 2][self.piece_position_y + 2]
                    == GridSquare::Moving
                && self.grid[self.piece_position_x + 2][self.piece_position_y + 1]
                    != GridSquare::Empty
                && self.grid[self.piece_position_x + 2][self.piece_position_y + 1]
                    != GridSquare::Moving
            {
                skip = true;
            }
            if self.piece_position_x + 2 < GRID_HORIZONTAL_SIZE
                && self.piece_position_y + 2 < GRID_VERTICAL_SIZE
                && self.grid[self.piece_position_x + 1][self.piece_position_y + 2]
                    == GridSquare::Moving
                && self.grid[self.piece_position_x + 2][self.piece_position_y + 2]
                    != GridSquare::Empty
                && self.grid[self.piece_position_x + 2][self.piece_position_y + 2]
                    != GridSquare::Moving
            {
                skip = true;
            }

            if !skip {
                // something
                aux = self.piece[0][0];
                self.piece[0][0] = self.piece[3][0];
                self.piece[3][0] = self.piece[3][3];
                self.piece[3][3] = self.piece[0][3];
                self.piece[0][3] = aux;

                // something
                aux = self.piece[1][0];
                self.piece[1][0] = self.piece[3][1];
                self.piece[3][1] = self.piece[2][3];
                self.piece[2][3] = self.piece[0][2];
                self.piece[0][2] = aux;

                // something
                aux = self.piece[2][0];
                self.piece[2][0] = self.piece[3][2];
                self.piece[3][2] = self.piece[1][3];
                self.piece[1][3] = self.piece[0][1];
                self.piece[0][1] = aux;

                aux = self.piece[1][1];
                self.piece[1][1] = self.piece[2][1];
                self.piece[2][1] = self.piece[2][2];
                self.piece[2][2] = self.piece[1][2];
                self.piece[1][2] = aux;
            }

            // then
            for j in (0..GRID_VERTICAL_SIZE - 2).rev() {
                for i in 1..GRID_HORIZONTAL_SIZE {
                    if self.grid[i][j] == GridSquare::Moving {
                        self.grid[i][j] = GridSquare::Empty;
                    }
                }
            }
            for i in 0..4 {
                for j in 0..4 {
                    if self.piece[i][j] == GridSquare::Moving {
                        let i = i + self.piece_position_x;
                        let j = j + self.piece_position_y;
                        if i < GRID_HORIZONTAL_SIZE && j < GRID_VERTICAL_SIZE {
                            self.grid[i][j] = GridSquare::Moving;
                        }
                    }
                }
            }
            true
        } else {
            false
        }
    }

    // collisions
    fn check_detection(&mut self) {
        for j in (0..=GRID_VERTICAL_SIZE - 2).rev() {
            // todo: definitely way more sensible way to do this
            for i in 1..GRID_HORIZONTAL_SIZE - 1 {
                // count each square of the line
                if self.grid[i][j] == GridSquare::Moving
                    && (self.grid[i][j + 1] == GridSquare::Full
                        || self.grid[i][j + 1] == GridSquare::Block)
                {
                    self.detection = true;
                }
            }
        }
    }

    fn check_completion(&mut self) {
        for j in (0..=GRID_VERTICAL_SIZE - 2).rev() {
            let mut calculator = 0;
            for i in 1..GRID_HORIZONTAL_SIZE - 1 {
                // count each square of the line
                if self.grid[i][j] == GridSquare::Full {
                    calculator += 1;
                }

                // check if we completed the whole line
                if calculator == GRID_HORIZONTAL_SIZE - 2 {
                    self.line_to_delete = true;
                    calculator = 0;

                    // mark the completed line
                    for z in 1..GRID_HORIZONTAL_SIZE - 1 {
                        self.grid[z][j] = GridSquare::Fading;
                    }
                }
            }
        }
    }

    fn delete_complete_lines(&mut self) {
        for j in (0..=GRID_VERTICAL_SIZE - 2).rev() {
            while self.grid[1][j] == GridSquare::Fading {
                for i in 1..GRID_HORIZONTAL_SIZE - 1 {
                    self.grid[i][j] = GridSquare::Empty;
                }
                for y in (j - 1..=0).rev() {
                    for x in 1..GRID_HORIZONTAL_SIZE {
                        if self.grid[x][y] == GridSquare::Full {
                            self.grid[x][y + 1] = GridSquare::Full;
                            self.grid[x][y] = GridSquare::Empty;
                        } else if self.grid[x][y] == GridSquare::Fading {
                            self.grid[x][y + 1] = GridSquare::Fading;
                            self.grid[x][y] = GridSquare::Empty;
                        }
                    }
                }
            }
        }
    }
}

// todo: document this stuff
fn main() -> color_eyre::eyre::Result<()> {
    // error
    color_eyre::install()?;
    // todo: seed rand w time?

    // init window
    let (mut rl, thread) = raylib::init().size(640, 480).title("Tetris").build();

    // todo: is this so
    rl.set_target_fps(60);

    let mut game = Game::new();

    // main loop
    while !rl.window_should_close() {
        game.update(&rl);
        game.draw(&mut rl, &thread);
    }

    Ok(())

    // todo:
    // - add a timestep based on delta time (no physics to simulate in tetris)
    // - add definitions for all of the blocks
    // hey this is a pretty fun project
}
