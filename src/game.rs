const RED: u32   = 0xFF0000;
const GREEN: u32 = 0x00FF00;
const BLUE: u32  = 0x0000FF;
const CYAN: u32  = 0x00FFFF;
const YELLOW: u32 = 0xFFFF00;
const ORANGE: u32 = 0xFFA500;
const PURPLE: u32 = 0x800080;
const LOCK_DELAY: u64 = 500;

use std::time::{Instant};

// Clone represents the general ability to duplicate a value. 
// Copy is a subset of Clone for types that can be bitwise copied. 
// Rust requires Copy types to also implement Clone for trait consistency.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TetrominoKind {I, O, T, J, L, S, Z}

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

// use std::process::Termination;
// use std::time::Instant;
use std::{ops::Add};
impl Add for Pos {
    type Output = Pos;
    fn add(self, other: Pos) -> Pos { Pos { x: self.x + other.x, y: self.y + other.y } }
}

use std::ops::Mul;

use crate::input::{ConstMotion, LockMgr, MotionState};
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

#[derive(Clone, Copy)]
pub struct Tetromino {
    pub kind: TetrominoKind,
    pos: Pos,
    rot: Rotation,
}

impl Tetromino {
    pub fn new(kind: TetrominoKind, pos: Pos) -> Self {
        // Field Init Shorthand: Field names can be omitted if they match the argument names.
        Self { kind, pos, rot: Rotation::R0 }
    }

    fn relative_cells(kind: TetrominoKind, rot: Rotation) -> [Pos;4]/*fixed length array on stack / no heap */ {
        use TetrominoKind::*;
        use Rotation::*;
        match kind {
            I => match rot {
                R0   => [Pos{x:-1, y:0}, Pos{x:0, y:0}, Pos{x:1, y:0}, Pos{x:2, y:0}],
                R90  => [Pos{x:1, y:-1}, Pos{x:1, y:0}, Pos{x:1, y:1}, Pos{x:1, y:2}],
                R180 => [Pos{x:-1, y:1}, Pos{x:0, y:1}, Pos{x:1, y:1}, Pos{x:2, y:1}],
                R270 => [Pos{x:0, y:-1}, Pos{x:0, y:0}, Pos{x:0, y:1}, Pos{x:0, y:2}],
            },
            O => {
                [Pos{x:0, y:0}, Pos{x:1, y:0}, Pos{x:0, y:1}, Pos{x:1, y:1}]
            },
            T => match rot {
                R0   => [Pos{x:-1, y:0}, Pos{x:0, y:0}, Pos{x:1, y:0}, Pos{x:0, y:-1}],
                R90  => [Pos{x:0, y:-1}, Pos{x:0, y:0}, Pos{x:1, y:0}, Pos{x:0, y:1}],
                R180 => [Pos{x:-1, y:0}, Pos{x:0, y:0}, Pos{x:1, y:0}, Pos{x:0, y:1}],
                R270 => [Pos{x:-1, y:0}, Pos{x:0, y:0}, Pos{x:0, y:-1}, Pos{x:0, y:1}],
            },
            J => match rot {
                R0   => [Pos{x:-1, y:-1}, Pos{x:-1, y:0}, Pos{x:0, y:0}, Pos{x:1, y:0}],
                R90  => [Pos{x:0, y:-1}, Pos{x:1, y:-1}, Pos{x:0, y:0}, Pos{x:0, y:1}],
                R180 => [Pos{x:-1, y:0}, Pos{x:0, y:0}, Pos{x:1, y:0}, Pos{x:1, y:1}],
                R270 => [Pos{x:0, y:-1}, Pos{x:0, y:0}, Pos{x:0, y:1}, Pos{x:-1, y:1}],
            },
            L => match rot {
                R0   => [Pos{x:-1, y:0}, Pos{x:0, y:0}, Pos{x:1, y:0}, Pos{x:1, y:-1}],
                R90  => [Pos{x:0, y:-1}, Pos{x:0, y:0}, Pos{x:0, y:1}, Pos{x:1, y:1}],
                R180 => [Pos{x:-1, y:1}, Pos{x:-1, y:0}, Pos{x:0, y:0}, Pos{x:1, y:0}],
                R270 => [Pos{x:-1, y:-1}, Pos{x:0, y:-1}, Pos{x:0, y:0}, Pos{x:0, y:1}],
            },
            S => match rot {
                R0   => [Pos{x:-1, y:0}, Pos{x:0, y:0}, Pos{x:0, y:-1}, Pos{x:1, y:-1}],
                R90  => [Pos{x:0, y:-1}, Pos{x:0, y:0}, Pos{x:1, y:0}, Pos{x:1, y:1}],
                R180 => [Pos{x:-1, y:1}, Pos{x:0, y:1}, Pos{x:0, y:0}, Pos{x:1, y:0}],
                R270 => [Pos{x:-1, y:-1}, Pos{x:-1, y:0}, Pos{x:0, y:0}, Pos{x:0, y:1}],
            },
            Z => match rot {
                R0   => [Pos{x:-1, y:-1}, Pos{x:0, y:-1}, Pos{x:0, y:0}, Pos{x:1, y:0}],
                R90  => [Pos{x:1, y:-1}, Pos{x:1, y:0}, Pos{x:0, y:0}, Pos{x:0, y:1}],
                R180 => [Pos{x:-1, y:0}, Pos{x:0, y:0}, Pos{x:0, y:1}, Pos{x:1, y:1}],
                R270 => [Pos{x:0, y:-1}, Pos{x:0, y:0}, Pos{x:-1, y:0}, Pos{x:-1, y:1}],
            },
        }
    }

    pub fn world_cells(&self) -> [Pos;4] {
        let rel = Self::relative_cells(self.kind/*Copy */, self.rot /*Copy*/);

    //     pub trait Add<Rhs = Self> {
    //     type Output;
    //     // Required method
    //     fn add(self, rhs: Rhs) -> Self::Output;
        // }

        // The Add Trait use 'self', which performs Copy or Move.
        // thus Pos must derive the Copy trait; otherwise it perform Move.
        [self.pos + rel[0], self.pos + rel[1], self.pos + rel[2], self.pos + rel[3]]
    }

    // &self is equivalent to self: &Self.
    // it is syntactic sugar
    // Original form: fn rotate_cw(self: &Self) -> Tetromino
    fn rotate_cw(&self) -> Tetromino {
        // ..*self
        // In Rust, the struct update syntax is used to create a new struct instance based on an
        // existing one, allowing you to explicitly set new values for some fileds while copying the remaining
        // fileds from the original instance.
        Tetromino { rot: self.rot.cw(), ..*self/*here we need to use deref because self is a &Self */ }
    }

    fn rotate_ccw(&self) -> Tetromino {
        Tetromino { rot: self.rot.ccw(), ..*self }
    }

} 


pub struct Board {
    // It is recommended to use i32 instead of u32 for members used in indexing and coordinate math.
    pub width: i32,
    pub height: i32,
    pub cells : Vec<Option<TetrominoKind>>,
}


impl Board {
    fn new(width:i32, height:i32) -> Self {
        Self { width, height, cells: vec![None; (width*height) as usize] }
    }
    fn try_place(&mut self, t:&Tetromino) -> bool
    {
        if !self.can_place(t) { return false; }
        t.world_cells().into_iter().for_each(|pos| self.set_occupied(pos, Some(t.kind)));
        true
    }


    // clears full lines and returns the score.
    fn check_clear(&mut self) -> usize {

        // better to create a new usize here
        let width = self.width as usize;


        // collect the remaining lines that are not full
        let new_cells:Vec<Option<TetrominoKind>> =
            self.cells.chunks(width) /*iterm = &[T] */
            .filter(|row /*filter will add & to &[T]*/ | {
                row.iter().any(|cell| cell.is_none())
            })
            .flatten()
            .copied() /*&Option to Option */
            .collect();

        let n_cleared_lines= self.height as usize - (new_cells.len()/width);
        // padding from the top
        let mut paddings:Vec<Option<TetrominoKind>>  = vec![None; width * n_cleared_lines];
        paddings.extend(new_cells);
        self.cells = paddings;
        n_cleared_lines as usize
    }

    fn can_place(&self, t:&Tetromino) -> bool {
        !t.world_cells().into_iter().any(|pos|self.is_occupied(pos))
    }

    fn is_occupied(&self, pos:Pos) -> bool {
        if pos.x < 0 || pos.y < 0 || pos.x >= self.width || pos.y >= self.height {
            return true;
        }
        self.cells[(pos.y * self.width + pos.x) as usize] != None
    }

    fn set_occupied(&mut self, pos: Pos, value: Option<TetrominoKind>) {
        if pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height {
            self.cells[(pos.y * self.width + pos.x) as usize] = value;
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

fn try_hard_drop(board:&Board, t:&Tetromino) -> Option<Tetromino> 
{
    let mut current_tetris = t.clone();
    loop {
        // if let Some(next_tetris) = try_down(&board, &current_tetris) {
        //     current_tetris = next_tetris;
        // } else {
        //     break Some(current_tetris)
        // }

        // or
        let Some(next_tetris) = try_down(&board, &current_tetris) else {
            break Some(current_tetris);
        };
        current_tetris = next_tetris;
    }
}

fn try_horizon(board:&Board, t:&Tetromino, is_left:bool) -> Option<Tetromino>
{

    let d = if is_left {Pos {x : -1, y : 0}} else {Pos {x:1, y:0}};
    let t2 = Tetromino { kind: t.kind, pos: t.pos + d, rot: t.rot };
    if board.can_place(&t2) {Some(t2)} else {None}
}







pub trait TetrisGenerator {
    fn next(&mut self, x: i32, y: i32) -> Tetromino;
}

pub struct GameState {
    pub current_tetris: Tetromino,
    pub shadow: Option<Tetromino>,
    pub shadow_out_of_date: bool,
    pub board: Board,
    debounce: Vec<MotionState>,
    gravity: ConstMotion,
    lock_mgr: LockMgr,
    tetris_generator: Box<dyn TetrisGenerator>,
    score:usize,
    game_over:bool,
}


#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum GameCommand {
    MoveLeft,
    MoveRight,
    SoftDrop,
    HardDrop,
    Rotate,
    None,
}

// factory pattern
pub fn create_new_game(width:i32, height:i32, now:Instant) -> GameState {
    GameState::new(width, height, now, Box::new(RandomGenerator::new()))
}

impl GameState {
    pub fn get_score(&self) -> usize {
        self.score
    }
    pub fn get_game_over(&self) -> bool {
        self.game_over
    }

    // what is mut generator ?
    // just like let mut generator = generator// re-binding
    // make it private, use factory create_new_game instead.
    fn new(width:i32, height:i32, now:Instant, mut generator:Box<dyn TetrisGenerator>) -> Self {
        let current_tetris = generator.next(width/2, 1);
        GameState {
            current_tetris,
            shadow:None,
            shadow_out_of_date :true,
            board: Board::new(width, height),
            gravity: ConstMotion::new(500, now),
            debounce: vec![
                MotionState::new(120, 80),
                MotionState::new(120, 80),
                MotionState::new(120, 120),
                MotionState::new(999999, 999999),
                MotionState::new(999999, 999999),
            ],
            lock_mgr: LockMgr::new(LOCK_DELAY),
            tetris_generator:generator,
            score:0,
            game_over:false,
        }
    }

    fn debounce_update(&mut self, is_press:bool, command:GameCommand, now:Instant) -> bool {
        use GameCommand::*;
        match command {
           MoveLeft =>  self.debounce[0].update(is_press, now),
           MoveRight => self.debounce[1].update(is_press, now),
           SoftDrop => self.debounce[2].update(is_press, now),
           HardDrop => self.debounce[3].update(is_press, now),
           Rotate => self.debounce[4].update(is_press, now),
           None => false
        }
    }


    fn is_game_over(&mut self) -> bool {
        if self.board.try_place(&self.current_tetris) {
            self.score += self.board.check_clear();
            self.current_tetris = self.tetris_generator.next(self.board.width/2, 1);
            self.shadow_out_of_date = true;
            false
        } else {
            true
        }
    }
    fn do_gravity(&mut self, now:Instant) -> Option<Tetromino> 
    {
        if !self.gravity.update(now) {
            return None;
        }
        try_down(&self.board, &self.current_tetris)
    }

    pub fn update(&mut self, press:bool, command:GameCommand, now:Instant) -> bool
    {
        if self.game_over {
            return false;
        }

        let can_acntion = self.debounce_update(press, command, now);

        use GameCommand::*;
        let moved_tetromino = 
            if can_acntion {
                match command {
                    MoveLeft =>  try_horizon(&self.board, &self.current_tetris, true),
                    MoveRight => try_horizon(&self.board, &self.current_tetris, false),
                    SoftDrop => try_down(&self.board, &self.current_tetris),
                    HardDrop => try_hard_drop(&self.board, &self.current_tetris),
                    Rotate => rotate_with_kick(&self.board, &self.current_tetris),
                    None => Option::None,
                }
            } else {
                Option::None
            };

        let mut res  = false;
        if let Some(next_pos) = moved_tetromino {
            self.current_tetris = next_pos;
            self.shadow_out_of_date = true;
            res = true;
        }

        if(GameCommand::HardDrop == command) && res {
            self.game_over =  self.is_game_over();
            self.lock_mgr.reset();
            return true; 
        }

        // check lock
        if try_down(&self.board, &self.current_tetris).is_none() {
            self.lock_mgr.start_if_not(now);
            if self.lock_mgr.lock(now) {
                self.lock_mgr.reset();
                self.game_over =  self.is_game_over();
            }
            return res;
        }
        // still can go down, unlock the locking time
        self.lock_mgr.reset();
        self.current_tetris = self.do_gravity(now).unwrap_or(self.current_tetris);
        res
    }

    fn update_press(&mut self,  command:GameCommand, now:Instant) -> bool
    {
        self.update(true, command, now)
    }

    pub fn get_shadow(&mut self) -> Option<Tetromino> {
        if self.shadow_out_of_date {
            self.shadow = try_hard_drop(&self.board, &self.current_tetris);
            self.shadow_out_of_date = false;
        }
        self.shadow
    }

    pub fn get_tetromino(&mut self) -> &Tetromino {
        &self.current_tetris
    }

    pub fn get_board(&mut self) -> &Board {
        &self.board
    }

}


use rand::prelude::*;

// 1. 定義 struct，讓它持有泛型 R
pub struct RandomGenerator<R: Rng> {
    rng: R,
}

// 2. 實作你的 TetrisGenerator Trait
impl<R: Rng> TetrisGenerator for RandomGenerator<R> {
    fn next(&mut self, x: i32, y: i32) -> Tetromino {
        // 使用 self.rng.gen_range (注意：rand 0.8+ 常用 gen_range)
        let shape = match self.rng.random_range(0..7) {
            0 => TetrominoKind::I,
            1 => TetrominoKind::O,
            2 => TetrominoKind::T,
            3 => TetrominoKind::J,
            4 => TetrominoKind::L,
            5 => TetrominoKind::S,
            _ => TetrominoKind::Z,
        };
        Tetromino::new(shape, Pos { x, y })
    }
}

// 3. 為了方便使用，提供一個建立方法
impl RandomGenerator<ThreadRng> {
    pub fn new() -> Self {
        Self {
            rng: rand::rng(),
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct MockGen {
    }
    impl TetrisGenerator for MockGen {
        fn next(&mut self, x:i32, y:i32) -> Tetromino {
            Tetromino { kind: TetrominoKind::I, pos:Pos{x,y}, rot: Rotation::R0 }
        }
    }
    impl MockGen {
        fn new() -> Self {
            Self{}
        }
    }

    #[test]
    fn test_game_init() {
        // let generator = Box::new(RandomGenerator::new());
        // let game = GameState::new_game(10, 10, Instant::now(), generator);
        let game = create_new_game(10, 10, Instant::now());
        assert_eq!(game.current_tetris.pos, Pos{x:5, y:1});
        assert_eq!(game.board.width, 10);
        assert_eq!(game.board.height, 10);
        assert_eq!(game.board.cells, vec![None; 100]);
    }

    #[test]
    fn test_tetris_move() {
        let init_time = Instant::now();
        let mut game = create_new_game(10, 10, init_time);
        assert_eq!(game.current_tetris.pos, Pos{x:5, y:1});
        game.update_press(GameCommand::MoveLeft, init_time);
        assert_eq!(game.current_tetris.pos, Pos{x:4, y:1});

        // debounce
        game.update_press(GameCommand::MoveLeft, init_time);
        assert_ne!(game.current_tetris.pos, Pos{x:3, y:1});

        // debounce pass
        game.update_press(GameCommand::MoveLeft, init_time + Duration::from_millis(121));
        assert_eq!(game.current_tetris.pos, Pos{x:3, y:1});
    }

    #[test]
    fn test_tetris_gravity() {
        let init_time = Instant::now();
        let mut game = create_new_game(10, 10, init_time);
        assert_eq!(game.current_tetris.pos, Pos{x:5, y:1});
        game.update_press(GameCommand::None, init_time + Duration::from_millis(500));
        assert_eq!(game.current_tetris.pos, Pos{x:5, y:1});
        game.update_press(GameCommand::None, init_time + Duration::from_millis(501));
        assert_eq!(game.current_tetris.pos, Pos{x:5, y:2});
        game.update_press(GameCommand::None, init_time + Duration::from_millis(1002));
        assert_eq!(game.current_tetris.pos, Pos{x:5, y:3});
    }

    #[test]
    fn test_hard_drop() {
        let init_time = Instant::now();
        let mut game = GameState::new(10, 10, init_time, Box::new(MockGen::new()));
        assert_eq!(game.current_tetris.pos, Pos{x:5, y:1});
        game.update_press(GameCommand::HardDrop, init_time);
        assert_ne!(game.board.cells[9*10+4], None);
        assert_ne!(game.board.cells[9*10+5], None);
        assert_ne!(game.board.cells[9*10+6], None);
        assert_ne!(game.board.cells[9*10+7], None);
    }

    #[test]
    fn test_soft_drop() {
        let init_time = Instant::now();
        let mut game = create_new_game(10, 10, init_time);
        assert_eq!(game.current_tetris.pos, Pos{x:5, y:1});
        game.update_press(GameCommand::SoftDrop, init_time);
        assert_eq!(game.current_tetris.pos, Pos{x:5, y:2});

        // + gravity
        game.update_press(GameCommand::SoftDrop, init_time + Duration::from_millis(501));
        assert_eq!(game.current_tetris.pos, Pos{x:5, y:4});
    }

    #[test]
    fn test_hard_drop_debounce() {
        let init_time = Instant::now();
        let mut game = GameState::new(10, 10, init_time, Box::new(MockGen::new()));
        game.update_press(GameCommand::SoftDrop, init_time);
        assert_eq!(game.current_tetris.pos, Pos{x:5, y:2});

        // trigger hard drop
        assert!(game.update_press(GameCommand::HardDrop, init_time));
        // create a new tetromino
        assert_eq!(game.current_tetris.pos, Pos{x:5, y:1});

        // won't trigger
        assert!(!game.update_press(GameCommand::HardDrop, init_time));
        assert_eq!(game.current_tetris.pos, Pos{x:5, y:1});

        // release
        assert!(!game.update(false, GameCommand::HardDrop, init_time));
        // can trigger
        assert!(game.update(true, GameCommand::HardDrop, init_time));
    }


    #[test]
    fn test_lock() {
        let init_time = Instant::now();
        let mut game = GameState::new(10, 3, init_time, Box::new(MockGen::new()));
        assert!(game.update_press(GameCommand::SoftDrop, init_time));
        assert_eq!(game.current_tetris.pos, Pos{x:5, y:2});

        // lock time is 500, we can still move left
        assert!(game.update_press(GameCommand::MoveLeft, init_time + Duration::from_millis(500)));
        assert_eq!(game.current_tetris.pos, Pos{x:4, y:2});

        // lock!
        assert!(!game.update_press(GameCommand::None, init_time + Duration::from_millis(501)));
        assert_ne!(game.board.cells[2*10+3], None);
        assert_ne!(game.board.cells[2*10+4], None);
        assert_ne!(game.board.cells[2*10+5], None);
        assert_ne!(game.board.cells[2*10+6], None);
        // new
        assert_eq!(game.current_tetris.pos, Pos{x:5, y:1});
    }

    #[test]
    fn test_bug_hard_drop_instant_game_over() {
        let init_time = Instant::now();
        let mut game = GameState::new(10, 5, init_time, Box::new(MockGen::new()));
        game.update(true, GameCommand::HardDrop, init_time);
        game.update(true, GameCommand::HardDrop, init_time);
        game.update(true, GameCommand::HardDrop, init_time);
        assert!(!game.get_game_over());
    }

}