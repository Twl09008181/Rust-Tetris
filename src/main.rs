use minifb::{Key, KeyRepeat, MouseButton, Window, WindowOptions};
use std::time::{Duration, Instant};


const BLACK: u32 = 0x000000;
const WHITE: u32 = 0xFFFFFF;
const RED: u32   = 0xFF0000;
const GREEN: u32 = 0x00FF00;
const BLUE: u32  = 0x0000FF;
const CYAN: u32  = 0x00FFFF;
const YELLOW: u32 = 0xFFFF00;
const ORANGE: u32 = 0xFFA500;
const PURPLE: u32 = 0x800080;

// --- 常量定義 ---
const WIDTH: usize = 300;
const HEIGHT: usize = 500;
const BLOCK_SIZE: i32 = 20; // Tetris 方塊的像素大小
const LOCK_DELAY:Duration = Duration::from_millis(500);

// --- 輔助函數：繪製單一方塊 ---
// 將一個 BLOCK_SIZE x BLOCK_SIZE 的方塊，繪製到緩衝區的 (x, y) 座標
fn draw_square(buffer: &mut [u32], x: i32, y: i32, color: u32) {
    // 簡單的邊界檢查
    if x < 0 || y < 0 || x + BLOCK_SIZE > WIDTH as i32 || y + BLOCK_SIZE > HEIGHT as i32 {
        // println!("out of boundary {x} {y}");
        return;
    }

    // 遍歷方塊的每個像素
    for row in 0..BLOCK_SIZE {
        for col in 0..BLOCK_SIZE {
            // 計算視窗中的實際像素坐標
            let px = (x + col) as usize;
            let py = (y + row) as usize;

            // 計算在 buffer 中的一維索引： index = y * WIDTH + x
            let index = py * WIDTH + px;
            
            // 安全檢查（確保索引在範圍內）並寫入顏色
            if index < buffer.len() {
                 buffer[index] = color;
            }
            // println!("{row} {col} = {color}");
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TetrominoKind {I, O, T, J, L, S, Z}

impl TetrominoKind {
    pub fn color(&self) -> u32 {
        use TetrominoKind::*;
        match self {
            I => RED, 
            O => GREEN,
            T => BLUE,
            J => CYAN,
            L => YELLOW,
            S => ORANGE,
            Z => PURPLE
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Rotation {R0, R90, R180, R270 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos { pub x: i32, pub y: i32 }

impl Rotation {
    fn cw(self) -> Rotation {
        use Rotation::*;
        match self {
            R0 => R90,
            R90 => R180,
            R180 => R270,
            R270 => R0
        }
    }
    fn ccw(self) -> Rotation {
        use Rotation::*;
        match self {
            R90 => R0,
            R180 => R90,
            R270 => R180,
            R0 => R270
        }
    }
}

use std::{ops::Add, process::Termination};
impl Add for Pos {
    type Output = Pos;
    fn add(self, other: Pos) -> Pos { Pos { x: self.x + other.x, y: self.y + other.y } }
}

use std::ops::Mul;
// 實作 Pos * i32
impl Mul<i32> for Pos {
    type Output = Pos;

    fn mul(self, rhs: i32) -> Pos {
        Pos {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
struct Tetromino {
    kind: TetrominoKind,
    pos: Pos,
    rot: Rotation,
}

impl Tetromino {
    pub fn new(kind: TetrominoKind, pos: Pos) -> Self {
        // Rust 允許在初始化 struct 時，如果 field 名稱跟變數名稱相同，可以省略 field: value 形式
        Self { kind, pos, rot: Rotation::R0 }
    }

    pub fn relative_cells(kind: TetrominoKind, rot: Rotation) -> [Pos;4]/*fixed length array on stack */ {
        use TetrominoKind::*;
        use Rotation::*;
        match kind {
            I => match rot {
                R0    => [Pos{x: -1, y:0}, Pos{x:0, y:0}, Pos{x:1, y:0}, Pos{x:2, y:0}],
                R90   => [Pos{x:1, y:-1}, Pos{x:1, y:0}, Pos{x:1, y:1}, Pos{x:1, y:2}],
                R180  => [Pos{x:-1, y:1}, Pos{x:0, y:1}, Pos{x:1, y:1}, Pos{x:2, y:1}],
                R270  => [Pos{x:0, y:-1}, Pos{x:0, y:0}, Pos{x:0, y:1}, Pos{x:0, y:2}],
            },
            O => { // O 塊通常不變形（偏移可視 engine 決定）
                [Pos{x:0,y:0}, Pos{x:1,y:0}, Pos{x:0,y:1}, Pos{x:1,y:1}]
            }
            T => match rot {
                R0    => [Pos{x:-1,y:0}, Pos{x:0,y:0}, Pos{x:1,y:0}, Pos{x:0,y:1}],
                R90   => [Pos{x:0,y:-1}, Pos{x:0,y:0}, Pos{x:0,y:1}, Pos{x:1,y:0}],
                R180  => [Pos{x:-1,y:0}, Pos{x:0,y:0}, Pos{x:1,y:0}, Pos{x:0,y:-1}],
                R270  => [Pos{x:0,y:-1}, Pos{x:0,y:0}, Pos{x:0,y:1}, Pos{x:-1,y:0}],
            },
            J => match rot {
                R0    => [Pos{x:-1,y:0}, Pos{x:0,y:0}, Pos{x:1,y:0}, Pos{x:-1,y:1}],
                R90   => [Pos{x:0,y:-1}, Pos{x:0,y:0}, Pos{x:0,y:1}, Pos{x:1,y:-1}],
                R180  => [Pos{x:-1,y:0}, Pos{x:0,y:0}, Pos{x:1,y:0}, Pos{x:1,y:-1}],
                R270  => [Pos{x:0,y:-1}, Pos{x:0,y:0}, Pos{x:0,y:1}, Pos{x:-1,y:1}],
            },
            L => match rot {
                R0    => [Pos{x:-1,y:0}, Pos{x:0,y:0}, Pos{x:1,y:0}, Pos{x:1,y:1}],
                R90   => [Pos{x:0,y:-1}, Pos{x:0,y:0}, Pos{x:0,y:1}, Pos{x:1,y:1}],
                R180  => [Pos{x:-1,y:0}, Pos{x:0,y:0}, Pos{x:1,y:0}, Pos{x:-1,y:-1}],
                R270  => [Pos{x:0,y:-1}, Pos{x:0,y:0}, Pos{x:0,y:1}, Pos{x:-1,y:-1}],
            },
            S => match rot {
                R0    => [Pos{x:-1,y:0}, Pos{x:0,y:0}, Pos{x:0,y:1}, Pos{x:1,y:1}],
                R90   => [Pos{x:0,y:-1}, Pos{x:0,y:0}, Pos{x:1,y:0}, Pos{x:1,y:1}],
                R180  => [Pos{x:-1,y:-1}, Pos{x:0,y:-1}, Pos{x:0,y:0}, Pos{x:1,y:0}],
                R270  => [Pos{x:-1,y:-1}, Pos{x:-1,y:0}, Pos{x:0,y:0}, Pos{x:0,y:1}],
            },
            Z => match rot {
                R0    => [Pos{x:-1,y:1}, Pos{x:0,y:1}, Pos{x:0,y:0}, Pos{x:1,y:0}],
                R90   => [Pos{x:1,y:-1}, Pos{x:1,y:0}, Pos{x:0,y:0}, Pos{x:0,y:1}],
                R180  => [Pos{x:-1,y:0}, Pos{x:0,y:0}, Pos{x:0,y:-1}, Pos{x:1,y:-1}],
                R270  => [Pos{x:0,y:-1}, Pos{x:0,y:0}, Pos{x:-1,y:0}, Pos{x:-1,y:1}],
            },
        }
    }
    pub fn world_cells(&self) -> [Pos;4] {
        // use Clone
        let rel = Self::relative_cells(self.kind, self.rot);
        // require impl Add for Pos
        // otherwise need 
        // no need to derive copy
        // [Pos{x:self.pos.x + rel[0].x,y:self.pos.y + rel[0].y} ,Pos{x:self.pos.x + rel[1].x,y:self.pos.y + rel[1].y}
        // ,Pos{x:self.pos.x + rel[2].x,y:self.pos.y + rel[2].y} ,Pos{x:self.pos.x + rel[3].x,y:self.pos.y + rel[3].y}]

        // need to derive copy
        [self.pos + rel[0], self.pos + rel[1], self.pos + rel[2], self.pos + rel[3]]
    }

    // sugar
    // fn rotate_cw(self: &Self) -> Tetromino
    fn rotate_cw(&self) -> Tetromino {
        Tetromino { rot: self.rot.cw(), ..*self }
    }
    // self is a reference
    fn rotate_ccw(&self) -> Tetromino {
        Tetromino { rot: self.rot.ccw(), ..*self }
    }

} 


struct Board {
    width: i32,
    height: i32,
    cells : Vec<Option<TetrominoKind>>,
}


impl Board {
    pub fn new(width:i32, height:i32) -> Self {
        Self { width, height, cells: vec![None; (width*height) as usize] }
    }
    pub fn try_place(&mut self, t:&Tetromino) -> bool
    {
        if !self.can_place(t) { return false; }
        t.world_cells().into_iter().for_each(|pos| self.set_occupied(pos, Some(t.kind)));
        true
    }


    pub fn check_clear(&mut self) -> usize {

        let mut clear_row = vec!();
        for row in 0..self.height {

            let mut good = true;
            for col in 0..self.width {
                if self.cells[ (row * self.width + col) as usize ] == None {
                    good = false;
                    break;
                }
            }
            if good {
                clear_row.push(row);
            }
        }
        for row in &clear_row {
            for col in 0..self.width {
                self.set_occupied(Pos{x:col, y:*row}, None);
            }
        }
        clear_row.len()
    }

    pub fn can_place(&self, t:&Tetromino) -> bool {
        !t.world_cells().into_iter().any(|pos|self.is_occupied(pos))
    }

    pub fn is_occupied(&self, pos:Pos) -> bool {
        if pos.x < 0 || pos.y < 0 || pos.x >= self.width || pos.y >= self.height {
            return true;
        }
        self.cells[(pos.y * self.width + pos.x) as usize] != None
    }

    pub fn set_occupied(&mut self, pos: Pos, value: Option<TetrominoKind>) {
        if pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height {
            self.cells[(pos.y * self.width + pos.x) as usize] = value;
        }
    }

    pub fn show(&self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = (row * self.width + col) as usize;
                print!("{}", if self.cells[idx] != None {"-"} else {"*"});
            }
            println!();
        }
    }
}


const KICKS : [Pos;5] = [
    Pos {x:0, y:0},
    Pos {x:-1, y:0},
    Pos {x:1, y:0},
    Pos {x:0, y:-1},
    Pos {x:0, y:1},
];

fn rotate_with_kick(board:&Board, t:&Tetromino) -> Option<Tetromino> 
{
    let rotated = t.rotate_cw();
    KICKS.iter().find_map(|&kick| {
        let kicked = Tetromino { pos:rotated.pos  + kick, ..rotated };
        if board.can_place(&kicked) {Some(kicked)} else {None}
    })
}

fn try_down(board:&Board, t:&Tetromino) -> Option<Tetromino>
{
    let d = Pos {x : 0, y : 1};
    let t2 = Tetromino { kind: t.kind, pos: t.pos + d, rot: t.rot };
    if board.can_place(&t2) {Some(t2)} else {None}
}

fn try_horizon(board:&Board, t:&Tetromino, is_left:bool) -> Option<Tetromino>
{

    let d = if is_left {Pos {x : -1, y : 0}} else {Pos {x:1, y:0}};
    let t2 = Tetromino { kind: t.kind, pos: t.pos + d, rot: t.rot };
    if board.can_place(&t2) {Some(t2)} else {None}
}


// fn main() {
//     let t = Tetromino::new(TetrominoKind::I,Pos{x:0, y:0});
//     let mut b = Board::new(10, 10);
//     if let Some(res) = rotate_with_kick(&b, &t) {
//         b.try_place(&res);
//         b.show();
//     }
// }



use rand::prelude::*;  // 引入 Rng trait
fn create_new_tetris() -> Tetromino 
{
    let mut rng = rand::rng();  // 取得隨機數生成器

    let shape = 
    match rng.random_range(0..7) {  // 產生 0~6
        0 => TetrominoKind::I,
        1 => TetrominoKind::O,
        2 => TetrominoKind::T,
        3 => TetrominoKind::J,
        4 => TetrominoKind::L,
        5 => TetrominoKind::S,
        _ => TetrominoKind::Z,
    };
    Tetromino::new(shape, Pos{x:WIDTH as i32 / 2 / BLOCK_SIZE, y:0})
}

fn draw_tertromino(buffer:&mut [u32], t:&Tetromino) {
    for pos in t.world_cells() {
        draw_square(buffer, pos.x * BLOCK_SIZE , pos.y * BLOCK_SIZE , t.kind.color());
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



// #[derive(PartialEq, Eq, Clone, Copy)]
// enum MoveStat {NPRESS, PRESS}
// struct Move {
//     key: minifb::Key,
//     last_record_time: Option<Instant>,
//     state: MoveStat,
// }

// impl Move {

//     fn new(key:minifb::Key) -> Self {
//         Move {
//             key, 
//             last_record_time:None,
//             state:MoveStat::NPRESS
//         }
//     }

//     fn do_move(&self, board:&Board, t:&Tetromino) -> Option<Tetromino> 
//     {
//         match self.key {
//             Key::Left => try_horizon(&board, &t, true),
//             Key::Right => try_horizon(&board, &t, false),
//             Key::Down => try_down(&board, &t),
//             Key::LeftCtrl => rotate_with_kick(&board,&t),
//             _ => None
//         }
//     }

//     fn update(&mut self, press:bool, board:&Board, t:&Tetromino) -> Option<Tetromino> 
//     {
//         if press == false {
//             self.last_record_time = None;
//             self.state = MoveStat::NPRESS;
//             return None;
//         }

//         // first time we press
//         let now = Instant::now();
//         if self.state == MoveStat::NPRESS {
//             self.last_record_time = Some(now);
//             self.state = MoveStat::PRESS;
//             return self.do_move(board, t);
//         }

//         // de-bounce + DAS
//         let move_delay = 
//         if self.key == Key::Left || self.key == Key::Right {
//             // DAS & ARR 
//             Duration::from_millis(80)
//         } else {
//             Duration::from_millis(200)
//         };

//         let press_pass = self.last_record_time.map_or(false, |t| now.duration_since(t) > move_delay);
//         if press_pass == false {
//             return None;
//         }
//         self.last_record_time = Some(now);
//         self.do_move(board, t)
//     }

// }


// struct InputController {



// }



#[derive(PartialEq, Eq, Clone, Copy)]
enum KeyState {NPRESS, PRESS, DAS}

#[derive(Clone, Copy)]
struct MotionConfig {
    das_delay: Duration,
    arr: Duration,
}


#[derive(Clone, Copy)]
struct MotionState {
    key_state : KeyState,
    last_time : Option<Instant>,
    config: MotionConfig,
}
impl MotionState {
    fn new(das_delay:u64, arr:u64) -> Self {
        Self {
            key_state : KeyState::NPRESS,
            last_time: None,
            config: MotionConfig {das_delay:Duration::from_millis(das_delay), arr:Duration::from_millis(arr)}
        }
    }

    fn update(&mut self, is_pressed:bool, current_time: Instant) -> bool
    {
        if !is_pressed {
            self.key_state = KeyState::NPRESS;
            self.last_time = None;
            return false;
        }

        match self.key_state {
            KeyState::NPRESS => {
                self.key_state = KeyState::PRESS;
                self.last_time = Some(current_time);
                true
            },
            KeyState::PRESS => {
                if let Some(last_time) = self.last_time {
                    if current_time.duration_since(last_time) > self.config.das_delay {
                        self.last_time = Some(current_time);
                        self.key_state = KeyState::DAS;
                        return true;
                    }
                }
                false
            },
            KeyState::DAS => {
                if let Some(last_time) = self.last_time {
                    if current_time.duration_since(last_time) > self.config.arr {
                        self.last_time = Some(current_time);
                        return true;
                    }
                }
                false
            }}
    }
}

struct ConstMotion {
    impl_motion : MotionState,
}

impl ConstMotion {
    fn new(delay:u64) -> Self {
        Self {
            impl_motion: MotionState::new(delay, delay)
        }
    }
    fn update(&mut self, current_time: Instant) -> bool {
        self.impl_motion.update(true, current_time)
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


    // 設置更新速率，限制為約 60 FPS
    window.set_target_fps(1000); 

    let mut current_tetris = create_new_tetris();
    let mut board = Board::new(WIDTH as i32 / BLOCK_SIZE, HEIGHT as i32 / BLOCK_SIZE);

    let mut motions = vec![MotionState::new(120, 30), MotionState::new(120, 30), MotionState::new(120, 120), MotionState::new(999999, 9999999)];

    let keys = vec![Key::Left, Key::Right, Key::Down, Key::LeftCtrl];

    let mut gravity = ConstMotion::new(500);
    let mut lock_time = None;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        for (&key, motion) in keys.iter().zip(motions.iter_mut()) {
            let do_move = motion.update(window.is_key_down(key), now);
            if do_move {
                current_tetris = match key {
                    Key::Left => try_horizon(&board, &current_tetris, true).unwrap_or(current_tetris),
                    Key::Right => try_horizon(&board, &current_tetris, false).unwrap_or(current_tetris),
                    Key::Down => try_down(&board, &current_tetris).unwrap_or(current_tetris),
                    Key::LeftCtrl => rotate_with_kick(&board,&current_tetris).unwrap_or(current_tetris),
                    _ => current_tetris,
                };
            }
        }



        let can_drop = try_down(&board, &current_tetris).is_some();
        if can_drop {
            lock_time = None;
            if gravity.update(now) {
                if let Some(next_tetris) = try_down(&board, &current_tetris) {
                    current_tetris = next_tetris;
                }
            }
        } else {
            match lock_time {
                None => {
                    lock_time = Some(now);
                },
                Some(t) => {
                    if now.duration_since(t) > LOCK_DELAY {
                        if board.try_place(&current_tetris) {
                            current_tetris = create_new_tetris();
                        }
                        lock_time = None;
                    }
                }
            }
        }

        buffer.fill(BLACK); // 設為黑色 
        draw_board(&mut buffer, &board);
        draw_tertromino(&mut buffer, &current_tetris);
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}