
// use is shortcut. the place we define the mod is in the lib.rs by using mod game and mod input
use minifb::{Key, Window, WindowOptions};
use std::time::{Instant};
use tetris::game::{Board, GameCommand, Tetromino, create_new_game};

// mod	Definition / Loading. Tells Rust to look for a file and include it in the compilation tree.	The Foundation. "I have a room called game."	namespace game { ... } or adding a file to your CMake project.
// pub	Visibility / Access. Determines if a module, function, or struct can be seen from the outside.	The Door Key. "This room is open to visitors."	public: in a class or a header file exposed in an API.
// use	Aliasing / Shortcut. Brings a path into the current scope so you don't have to type the full name.	The Teleport. "Call 'the room in the back' just 'Kitchen'."


// --- 常量定義 ---
const BLACK: u32 = 0x000000;
const WIDTH: usize = 300;
const HEIGHT: usize = 500;
const BLOCK_SIZE: i32 = 20; // Tetris 方塊的像素大小

fn draw_tertromino(buffer:&mut [u32], t:&Tetromino) {
    for pos in t.world_cells() {
        draw_square(buffer, pos.x * BLOCK_SIZE , pos.y * BLOCK_SIZE , t.kind.color());
    }
}

fn draw_tertromino_with_color(buffer:&mut [u32], t:&Tetromino, color:u32) {
    for pos in t.world_cells() {
        draw_square(buffer, pos.x * BLOCK_SIZE , pos.y * BLOCK_SIZE , color);
    }
}

fn draw_board(buffer:&mut [u32], b:&Board) {
    for y in 0..b.height {
        for x in 0..b.width {
            if let Some(kind) = b.cells[(y * b.width + x) as usize] {
                draw_square(buffer, x * BLOCK_SIZE, y * BLOCK_SIZE, kind.color());
            }
        }
    }
}

// render
fn draw_square(buffer: &mut [u32], x: i32, y: i32, color: u32) {
    // 簡單的邊界檢查
    if x < 0 || y < 0 || x + BLOCK_SIZE > WIDTH as i32 || y + BLOCK_SIZE > HEIGHT as i32 {
        return;
    }

    for row in 0..BLOCK_SIZE {
        for col in 0..BLOCK_SIZE {
            let px = (x + col) as usize;
            let py = (y + row) as usize;
            let index = py * WIDTH + px;
            if index < buffer.len() {
                 buffer[index] = color;
            }
        }
    }
}




fn main() {

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Rust Tetris",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("window creation fail: {}", e);
    });


    // 60fps update
    window.set_target_fps(60); 

    let keys = vec![Key::Left, Key::Right, Key::Down, Key::LeftCtrl, Key::Space];


    let mut game = 
        create_new_game(WIDTH as i32 / BLOCK_SIZE, HEIGHT as i32 / BLOCK_SIZE, Instant::now());

    while window.is_open() && !window.is_key_down(Key::Escape) {

        if game.get_game_over() {
            window.update(); // otherwise I cannot read the Escape
            continue;
        }


        let now = Instant::now();

        for &key in keys.iter() {
            use GameCommand::*;
            let command = match key {
                Key::Left => MoveLeft, 
                Key::Right => MoveRight,
                Key::Down => SoftDrop,
                Key::LeftCtrl => Rotate,
                Key::Space => HardDrop,
                _ => None
            };
            game.update(window.is_key_down(key), command, now);
        }

        let shadow = game.get_shadow();
        buffer.fill(BLACK); // clean all 
        draw_board(&mut buffer, &game.get_board());
        if let Some(shadow) = shadow {
            draw_tertromino_with_color(&mut buffer, &shadow, 0x444444); // draw shadow first
        }
        draw_tertromino(&mut buffer, &game.current_tetris);

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}