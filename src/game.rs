use std::io::{Read, Result, Write};
use std::thread;
use std::time::Duration;

use termion::{clear, cursor, style};
use termion::color::{Bg, Blue, Color, Cyan, Green, Red, Rgb, Yellow};

use crate::graphics::*;
use termion::input::TermRead;
use termion::event::Key;
use termion::input::Keys;

pub struct Game<R: Read, W: Write> {
    x: u16,
    y: u16,
    score: u16,
    lines: u16,
    board: Board,
    stdin: Keys<R>,
    stdout: W,
    current_tetrimino: CurrentTetrimino,
    elapsed: Duration,
    fall_rate: Duration,
}

struct CurrentTetrimino {
    tetrimino: Tetrimino,
    x: u16,
    y: u16,
}

const LEFT_PANEL_WIDTH: u16 = 17;

const SCORE_WINDOW_HEIGHT: u16 = 8;
const HELP_WINDOW_HEIGHT: u16 = 12;

const BOARD_WIDTH: u16 = 10;
const BOARD_HEIGHT: u16 = 20;


impl<R: Read, W: Write> Game<R, W> {
    pub fn new(x: u16, y: u16, r: R, w: W) -> Self {
        Game {
            x,
            y,
            score: 0,
            lines: 0,
            board: Board::new(),
            stdin: r.keys(),
            stdout: w,
            current_tetrimino: Self::next_tetrimino(),
            elapsed: Duration::from_millis(0),
            fall_rate: Duration::from_millis(500),
        }
    }

    pub fn start(&mut self) -> Result<()> {
        write!(&mut self.stdout, "{}{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1), cursor::Hide)?;

        loop {
            thread::sleep(Duration::from_millis(50));
            match self.stdin.next() {
                Some(Ok(key)) => {
                    self.handle_key_input(key)
                }
                _ => {}
            }

            self.try_fuse_with_ground();
            self.draw_player_score()?;
            self.draw_help()?;
            self.draw_board()?;
            self.stdout.flush()?;

            self.update(Duration::from_millis(50));
        }
    }

    fn update(&mut self, elapsed: Duration) {
        self.elapsed += elapsed;

        if self.elapsed >= self.fall_rate {
            self.elapsed -= self.fall_rate;
            self.current_tetrimino.y += 1;
        }
    }

    fn handle_key_input(&mut self, key: Key) {
        match key {
            Key::Char('j') | Key::Left => {
                self.handle_tetrimino_move(-1, 0);
            }
            Key::Char('l') | Key::Right => {
                self.handle_tetrimino_move(1, 0);
            }
            Key::Char('k') | Key::Down => {}
            Key::Char('x') | Key::Char('z') => {}
            Key::Char('c') => {}
            _ => {}
        }
    }

    fn handle_tetrimino_move(&mut self, dx: isize, dy: isize) {
        // might overflow
        if self.can_fit_tetrimino(self.current_tetrimino.x as isize + dx,
                                  self.current_tetrimino.y as isize + dy,
                                  self.current_tetrimino.tetrimino.to_block()) {
            self.current_tetrimino.x = (self.current_tetrimino.x as isize + dx) as u16;
        }
    }

    fn can_fit_tetrimino(&self, x: isize, y: isize, block: &[[u8; 4]; 4]) -> bool {
        for (y, row) in block.iter().enumerate() {
            for (x, col) in row.iter().enumerate() {
                if *col == 1 {
                    // actual co-ordinates on the board
                    let x = x + x;
                    let y = y + y;

                    if x >= BOARD_WIDTH as usize {
                        return false;
                    } else if y >= BOARD_HEIGHT as usize {
                        return false;
                    } else if let Block::Occupied(_) = self.board.blocks[y][x] {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn try_fuse_with_ground(&mut self) {
        let tetrimino_block = self.current_tetrimino.tetrimino.to_block();

        let mut should_fuse = false;

        for (y, row) in tetrimino_block.iter().enumerate() {
            for (x, col) in row.iter().enumerate() {
                if *col == 1 && self.should_fuse_with_ground(x, y) {
                    should_fuse = true;
                    break;
                }
            }
        }

        if should_fuse {
            for (y, row) in tetrimino_block.iter().enumerate() {
                for (x, col) in row.iter().enumerate() {
                    if *col == 1 {
                        let rgb = self.current_tetrimino.tetrimino.to_color();
                        let x = self.current_tetrimino.x as usize + x;
                        let y = self.current_tetrimino.y as usize + y;
                        self.board.blocks[y][x] = Block::Occupied(rgb);
                    }
                }
            }

            self.current_tetrimino = Self::next_tetrimino()
        }
    }

    // check whether current tetrimino should be fused with ground if tetrimino_block[y][x] == 1
    fn should_fuse_with_ground(&self, x: usize, y: usize) -> bool {
        let x = self.current_tetrimino.x as usize + x;
        let next_y = self.current_tetrimino.y as usize + y + 1;

        if next_y == BOARD_HEIGHT as usize {
            return true;
        }

        match self.board.blocks.get(next_y) {
            Some(row) => {
                if let Some(Block::Occupied(_)) = row.get(x) {
                    true
                } else {
                    false
                }
            }
            _ => false
        }
    }

    fn draw_player_score(&mut self) -> Result<()> {
        let (x, y) = self.player_score_xy();
        create_window(&mut self.stdout, x, y, LEFT_PANEL_WIDTH, SCORE_WINDOW_HEIGHT)?;
        write!(self.stdout, "{}Score", cursor::Goto(x + 6, y + 2))?;
        write!(self.stdout, "{}score: {:04} ", cursor::Goto(x + 3, y + 4), self.score)?;
        write!(self.stdout, "{}lines: {:04} ", cursor::Goto(x + 3, y + 5), self.lines)
    }

    fn draw_help(&mut self) -> Result<()> {
        let (x, y) = self.help_window_xy();
        create_window(&mut self.stdout, x, y, LEFT_PANEL_WIDTH, HELP_WINDOW_HEIGHT)?;
        write!(self.stdout, "{}Ctrls", cursor::Goto(x + 6, y + 2))?;
        write!(self.stdout, "{}left   j, ←", cursor::Goto(x + 3, y + 4))?;
        write!(self.stdout, "{}right  l, →", cursor::Goto(x + 3, y + 5))?;
        write!(self.stdout, "{}drop   k, ↓", cursor::Goto(x + 3, y + 6))?;
        write!(self.stdout, "{}rotate x, z", cursor::Goto(x + 3, y + 7))?;
        write!(self.stdout, "{}hold   c", cursor::Goto(x + 3, y + 8))
    }

    fn draw_board(&mut self) -> Result<()> {
        let (x, y) = self.tetris_board_xy();
        create_window(&mut self.stdout, x, y, (BOARD_WIDTH * 2) + 2, BOARD_HEIGHT + 2)?;

        let (init_x, init_y) = self.tetris_board_xy();
        let mut y = init_y + 1;

        // draw the board
        for (y, row) in self.board.blocks.iter().enumerate() {
            for (x, col) in row.iter().enumerate() {
                match col {
                    Block::Free => {
                        write!(self.stdout, "{}{}  ", cursor::Goto(init_x + (x * 2) as u16 + 1, init_y + y as u16 + 1), style::Reset)?;
                    }
                    Block::Occupied(rgb) => {
                        write!(self.stdout, "{}{}  ", cursor::Goto(init_x + (x * 2) as u16 + 1, init_y + y as u16 + 1), Bg(*rgb))?;
                    }
                }
            }
        }

        // draw current tetrimino
        let tetrimino_block = self.current_tetrimino.tetrimino.to_block();
        for (y, row) in tetrimino_block.iter().enumerate() {
            for (x, col) in row.iter().enumerate() {
                if *col == 1 {
                    let x = init_x + (x * 2) as u16 + (self.current_tetrimino.x * 2) + 1;
                    let y = init_y + y as u16 + self.current_tetrimino.y + 1;
                    let tetrimino_color = self.current_tetrimino.tetrimino.to_color();
                    write!(self.stdout, "{}{}  ", cursor::Goto(x, y), Bg(tetrimino_color))?;
                }
            }
        }

        write!(self.stdout, "{}", style::Reset)
    }

    fn next_tetrimino() -> CurrentTetrimino {
        let next_type = TetriminoType::L;
        let (next_x, next_y) = Self::apply_initial_displacement(&next_type, 3, 0);
        let current_tetrimino = CurrentTetrimino {
            tetrimino: Tetrimino::new(next_type),
            x: next_x,
            y: next_y,
        };

        current_tetrimino
    }

    fn apply_initial_displacement(tetrimino_type: &TetriminoType, x: u16, y: u16) -> (u16, u16) {
        match tetrimino_type {
            TetriminoType::I => (x, y - 1),
            _ => (x, y)
        }
    }

    // placement methods
    fn player_score_xy(&self) -> (u16, u16) {
        (self.x, self.y)
    }

    fn help_window_xy(&self) -> (u16, u16) {
        (self.x, self.y + SCORE_WINDOW_HEIGHT + 2)
    }

    fn tetris_board_xy(&self) -> (u16, u16) {
        (self.x + LEFT_PANEL_WIDTH + 1, self.y)
    }
}

pub struct Board {
    blocks: [[Block; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize]
}

#[derive(Copy, Clone)]
pub enum Block {
    Free,
    Occupied(Rgb),
}

impl Board {
    fn new() -> Self {
        let blocks = [[Block::Free; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];
        Board { blocks }
    }
}

enum TetriminoType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

struct Tetrimino {
    tetrimino_type: TetriminoType,
    state: usize,
}

impl Tetrimino {
    pub fn new(tetrimino_type: TetriminoType) -> Self {
        Tetrimino { tetrimino_type, state: 0 }
    }

    pub fn to_color(&self) -> Rgb {
        match self.tetrimino_type {
            TetriminoType::I => Rgb(0, 255, 255),
            TetriminoType::O => Rgb(255, 255, 0),
            TetriminoType::T => Rgb(128, 0, 128),
            TetriminoType::S => Rgb(0, 128, 0),
            TetriminoType::Z => Rgb(255, 0, 0),
            TetriminoType::J => Rgb(0, 0, 255),
            TetriminoType::L => Rgb(255, 165, 0),
        }
    }

    pub fn to_block(&self) -> &[[u8; 4]; 4] {
        match self.tetrimino_type {
            TetriminoType::I => {
                [
                    [
                        [0, 0, 0, 0],
                        [1, 1, 1, 1],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [0, 0, 1, 0],
                        [0, 0, 1, 0],
                        [0, 0, 1, 0],
                        [0, 0, 1, 0],
                    ],
                    [
                        [0, 0, 0, 0],
                        [0, 0, 0, 0],
                        [1, 1, 1, 1],
                        [0, 0, 0, 0],
                    ],
                    [
                        [0, 1, 0, 0],
                        [0, 1, 0, 0],
                        [0, 1, 0, 0],
                        [0, 1, 0, 0],
                    ]
                ].get(self.state).unwrap()
            }
            TetriminoType::O => {
                &[
                    [0, 1, 1, 0],
                    [0, 1, 1, 0],
                    [0, 0, 0, 0],
                    [0, 0, 0, 0],
                ]
            }
            TetriminoType::T => {
                [
                    [
                        [0, 1, 0, 0],
                        [1, 1, 1, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [0, 1, 0, 0],
                        [0, 1, 1, 0],
                        [0, 1, 0, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [0, 0, 0, 0],
                        [1, 1, 1, 0],
                        [0, 1, 0, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [0, 1, 0, 0],
                        [1, 1, 0, 0],
                        [0, 1, 0, 0],
                        [0, 0, 0, 0],
                    ]
                ].get(self.state).unwrap()
            }
            TetriminoType::S => {
                [
                    [
                        [0, 1, 1, 0],
                        [1, 1, 0, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [0, 1, 0, 0],
                        [0, 1, 1, 0],
                        [0, 0, 1, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [0, 0, 0, 0],
                        [0, 1, 1, 0],
                        [1, 1, 0, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [1, 0, 0, 0],
                        [1, 1, 0, 0],
                        [0, 1, 1, 0],
                        [0, 0, 0, 0],
                    ]
                ].get(self.state).unwrap()
            }
            TetriminoType::Z => {
                [
                    [
                        [1, 1, 0, 0],
                        [0, 1, 1, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [0, 0, 1, 0],
                        [0, 1, 1, 0],
                        [0, 1, 0, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [0, 0, 0, 0],
                        [1, 1, 0, 0],
                        [0, 1, 1, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [0, 1, 0, 0],
                        [1, 1, 0, 0],
                        [1, 0, 0, 0],
                        [0, 0, 0, 0],
                    ]
                ].get(self.state).unwrap()
            }
            TetriminoType::J => {
                [
                    [
                        [1, 0, 0, 0],
                        [1, 1, 1, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [0, 1, 1, 0],
                        [0, 1, 0, 0],
                        [0, 1, 0, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [0, 0, 0, 0],
                        [1, 1, 1, 0],
                        [0, 0, 1, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [0, 1, 0, 0],
                        [0, 1, 0, 0],
                        [1, 1, 0, 0],
                        [0, 0, 0, 0],
                    ]
                ].get(self.state).unwrap()
            }
            TetriminoType::L => {
                [
                    [
                        [0, 0, 1, 0],
                        [1, 1, 1, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [0, 1, 0, 0],
                        [0, 1, 0, 0],
                        [0, 1, 1, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [1, 1, 1, 0],
                        [1, 0, 0, 0],
                        [0, 0, 0, 0],
                        [0, 0, 0, 0],
                    ],
                    [
                        [1, 1, 0, 0],
                        [0, 1, 0, 0],
                        [0, 1, 0, 0],
                        [0, 0, 0, 0],
                    ]
                ].get(self.state).unwrap()
            }
        }
    }

    pub fn rotate_clockwise(&mut self) {
        if self.state == 0 {
            self.state = 3
        } else {
            self.state -= 1;
        }
    }

    pub fn rotate_counter_clockwise(&mut self) {
        if self.state == 3 {
            self.state = 0
        } else {
            self.state += 1
        }
    }
}