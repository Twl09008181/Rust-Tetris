# Rust Tetris
A classic Tetris implementation written in Rust, focusing on strong encapsulation and precise input handling (DAS/ARR).

## 🚀 Project Highlights
Hybrid Architecture: Uses a main.rs (Binary) and lib.rs (Library) structure to decouple game logic from the rendering engine.

Precise Control (DAS/ARR): Built-in state machines for Delayed Auto Shift (DAS) and Auto Repeat Rate (ARR) to provide professional-grade handling.

Strict Encapsulation: Leverages Rust’s pub(crate) and private module system to hide internal states (like frame counters and generators), exposing only necessary APIs to the renderer.

Polymorphic Piece Generation: Utilizes Box<dyn TetrisGenerator> for a flexible piece queue system, allowing for random or seeded generation.

## ️ Tech Stack
Language: Rust 2021 Edition

Rendering: minifb (Cross-platform raw pixel buffer)

## Core Components:

GameState: Manages the game loop, line clears, and score.

MotionState: Handles key debouncing and continuous movement logic.

Board: Grid system with collision detection.

## File Structure
src/
├── main.rs          # Entry point: handles 60FPS loop and minifb window
├── lib.rs           # Crate Root: defines the module tree and public factory
├── game.rs          # Core Logic: tetromino movement and shadow calculation
└── input.rs         # (Private) Internal state machines for DAS/ARR and Lock Delay


## Control
Key,Action
Left / Right,Move (Supports DAS long-press for rapid shift)
Down,Soft Drop
Space,Hard Drop
Left Ctrl,Rotate Piece
Esc,Exit Game