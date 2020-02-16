use std::io::{Read, Result, Write};
use std::thread;
use std::time::Duration;

use termion::{clear, cursor, style};
use termion::color::{Bg, Rgb};

use crate::graphics::*;
use termion::input::TermRead;
use termion::event::Key;
use termion::input::Keys;
use termion::raw::{IntoRawMode, RawTerminal};

pub struct Game<R: Read, W: Write> {
    x: u16,
    y: u16,
    score: u16,
    lines: u16,
    board: Board,
    stdin: Keys<R>,
    stdout: W,
    current_tetrimino: CurrentTetrimino,
    next_tetrimino: TetriminoType,
    held_tetrimino: Option<TetriminoType>,
    can_hold: bool,
    elapsed: Duration,
    fall_rate: Duration,
}

struct CurrentTetrimino {
    tetrimino: Tetrimino,

    // These must be isize because tetrimino's grid can go out of bound
    // e.g block L has this shape on the left.
    //
    // 0 1 0 0      If this block is at the far left corner
    // 0 1 0 0      the x will be -1, doesn't have a good solution for this yet
    // 0 1 1 1      And this is why there are a lot of type casts in the code
    // 0 0 0 0      Not great
    x: isize,
    y: isize,
}

const LEFT_PANEL_WIDTH: u16 = 17;

const SCORE_WINDOW_HEIGHT: u16 = 8;
const HELP_WINDOW_HEIGHT: u16 = 12;

const BOARD_WIDTH: u16 = 10;
const BOARD_HEIGHT: u16 = 20;

const RIGHT_PANEL_WIDTH: u16 = 12;
const NEXT_WINDOW_HEIGHT: u16 = 10;
const HELD_WINDOW_HEIGHT: u16 = 10;


impl<R: Read, W: Write> Game<R, W> {
    pub fn new(x: u16, y: u16, r: R, w: W) -> Game<R, RawTerminal<W>> {
        Game {
            x,
            y,
            score: 0,
            lines: 0,
            board: Board::new(),
            stdin: r.keys(),
            stdout: w.into_raw_mode().unwrap(),
            current_tetrimino: Self::next_tetrimino(),
            next_tetrimino: Self::generate_tetrimino(),
            held_tetrimino: None,
            can_hold: true,
            elapsed: Duration::from_millis(0),
            fall_rate: Duration::from_millis(500),
        }
    }

    pub fn start(&mut self) -> Result<()> {
        write!(&mut self.stdout, "{}{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1), cursor::Hide)?;

        'main: loop {
            thread::sleep(Duration::from_millis(50));
            match self.stdin.next() {
                Some(Ok(key)) => {
                    match key {
                        Key::Char('j') | Key::Left => {
                            self.handle_tetrimino_move(-1, 0);
                        }
                        Key::Char('l') | Key::Right => {
                            self.handle_tetrimino_move(1, 0);
                        }
                        Key::Char('k') | Key::Down => {
                            self.handle_tetrimino_move(0, 1);
                        }
                        Key::Char('x') => {
                            self.current_tetrimino.tetrimino.rotate_clockwise();
                            if !self.can_fit_tetrimino(self.current_tetrimino.x,
                                                       self.current_tetrimino.y,
                                                       self.current_tetrimino.tetrimino.to_block()) {
                                self.current_tetrimino.tetrimino.rotate_counter_clockwise();
                            }
                        }
                        Key::Char('z') => {
                            self.current_tetrimino.tetrimino.rotate_counter_clockwise();
                            if !self.can_fit_tetrimino(self.current_tetrimino.x,
                                                       self.current_tetrimino.y,
                                                       self.current_tetrimino.tetrimino.to_block()) {
                                self.current_tetrimino.tetrimino.rotate_clockwise();
                            }
                        }
                        Key::Char('c') => {
                            self.try_hold_tetrimino();
                        }
                        Key::Char('q') => break 'main,
                        _ => {}
                    }
                }
                _ => {}
            }

            self.try_fuse_with_ground();
            self.draw_player_score()?;
            self.draw_help()?;
            self.draw_board()?;
            self.draw_next()?;
            self.draw_held()?;
            self.stdout.flush()?;

            self.update(Duration::from_millis(50));
        }
        write!(self.stdout, "{}{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1), cursor::Show)?;

        Ok(())
    }

    fn update(&mut self, elapsed: Duration) {
        self.elapsed += elapsed;

        if self.elapsed >= self.fall_rate {
            self.elapsed -= self.fall_rate;
            self.current_tetrimino.y += 1;
        }
    }

    fn try_hold_tetrimino(&mut self) {
        if !self.can_hold {
            return;
        }

        if let Some(current) = self.held_tetrimino.take() {
            self.held_tetrimino = Some(self.current_tetrimino.tetrimino.tetrimino_type);
            self.current_tetrimino = Self::next_tetrimino_with_type(current);
        } else {
            self.held_tetrimino = Some(self.current_tetrimino.tetrimino.tetrimino_type);
            self.current_tetrimino = Self::next_tetrimino();
        }

        self.can_hold = false;
    }

    fn handle_tetrimino_move(&mut self, dx: isize, dy: isize) {
        let new_x = self.current_tetrimino.x as isize + dx;
        let new_y = self.current_tetrimino.y as isize + dy;

        if self.can_fit_tetrimino(new_x, new_y, self.current_tetrimino.tetrimino.to_block()) {
            self.current_tetrimino.x = new_x;
            self.current_tetrimino.y = new_y;
        }
    }

    fn can_fit_tetrimino(&self, block_x: isize, block_y: isize, block: &[[u8; 4]; 4]) -> bool {
        for (y, row) in block.iter().enumerate() {
            for (x, col) in row.iter().enumerate() {
                if *col == 1 {
                    // actual co-ordinates on the board
                    let x = block_x + x as isize;
                    let y = block_y + y as isize;

                    if x >= BOARD_WIDTH as isize || x < 0 {
                        return false;
                    } else if y >= BOARD_HEIGHT as isize || y < 0 {
                        return false;
                    } else if let Block::Occupied(_) = self.board.blocks[y as usize][x as usize] {
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
                if *col == 1 && self.should_fuse_with_ground(x as isize, y as isize) {
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
                        let x = self.current_tetrimino.x + x as isize;
                        let y = self.current_tetrimino.y + y as isize;
                        self.board.blocks[y as usize][x as usize] = Block::Occupied(rgb);
                    }
                }
            }

            self.current_tetrimino = Self::next_tetrimino_with_type(self.next_tetrimino);
            self.next_tetrimino = Self::generate_tetrimino();
            self.can_hold = true;
        }
    }

    // check whether current tetrimino should be fused with ground if tetrimino_block[y][x] == 1
    fn should_fuse_with_ground(&self, x: isize, y: isize) -> bool {
        let x = self.current_tetrimino.x + x;
        let next_y = self.current_tetrimino.y + y + 1;

        if next_y == BOARD_HEIGHT as isize {
            return true;
        }

        match self.board.blocks.get(next_y as usize) {
            Some(row) => {
                if let Some(Block::Occupied(_)) = row.get(x as usize) {
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
        write!(self.stdout, "{}hold   c", cursor::Goto(x + 3, y + 8))?;
        write!(self.stdout, "{}quit   q", cursor::Goto(x + 3, y + 9))
    }

    fn draw_board(&mut self) -> Result<()> {
        let (x, y) = self.tetris_board_xy();
        create_window(&mut self.stdout, x, y, (BOARD_WIDTH * 2) + 2, BOARD_HEIGHT + 2)?;

        let (init_x, init_y) = self.tetris_board_xy();

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
                    let x = init_x as isize + x as isize * 2 + self.current_tetrimino.x * 2 + 1;
                    let y = init_y as isize + y as isize + self.current_tetrimino.y + 1;
                    let tetrimino_color = self.current_tetrimino.tetrimino.to_color();
                    write!(self.stdout, "{}{}  ", cursor::Goto(x as u16, y as u16), Bg(tetrimino_color))?;
                }
            }
        }

        write!(self.stdout, "{}", style::Reset)
    }

    fn draw_next(&mut self) -> Result<()> {
        let (win_x, win_y) = self.next_window_xy();
        create_window(&mut self.stdout, win_x, win_y, RIGHT_PANEL_WIDTH, NEXT_WINDOW_HEIGHT)?;
        write!(self.stdout, "{}Next", cursor::Goto(win_x + 4, win_y + 2))?;

        let block = self.next_tetrimino.get_block(0);
        for (y, row) in block.iter().enumerate() {
            for (x, col) in row.iter().enumerate() {
                if *col == 1 {
                    let x = win_x + x as u16 * 2 + 2;
                    let y = win_y + y as u16 + 4;
                    let color = self.next_tetrimino.get_color();
                    write!(self.stdout, "{}{}  ", cursor::Goto(x, y), Bg(color))?;
                }
            }
        }

        write!(self.stdout, "{}", style::Reset)
    }

    fn draw_held(&mut self) -> Result<()> {
        let (win_x, win_y) = self.held_window_xy();
        create_window(&mut self.stdout, win_x, win_y, RIGHT_PANEL_WIDTH, HELD_WINDOW_HEIGHT)?;
        write!(self.stdout, "{}Held", cursor::Goto(win_x + 4, win_y + 2))?;

        if let Some(held) = self.held_tetrimino {
            let block = held.get_block(0);
            for (y, row) in block.iter().enumerate() {
                for (x, col) in row.iter().enumerate() {
                    if *col == 1 {
                        let x = win_x + x as u16 * 2 + 2;
                        let y = win_y + y as u16 + 4;
                        let color = held.get_color();
                        write!(self.stdout, "{}{}  ", cursor::Goto(x, y), Bg(color))?;
                    }
                }
            }
        }

        write!(self.stdout, "{}", style::Reset)
    }

    fn generate_tetrimino() -> TetriminoType {
        TetriminoType::L
    }

    fn next_tetrimino() -> CurrentTetrimino {
        let next_type = Self::generate_tetrimino();
        Self::next_tetrimino_with_type(next_type)
    }

    fn next_tetrimino_with_type(ttype: TetriminoType) -> CurrentTetrimino {
        let (next_x, next_y) = Self::apply_initial_displacement(&ttype, 3, 0);
        let current_tetrimino = CurrentTetrimino {
            tetrimino: Tetrimino::new(ttype),
            x: next_x,
            y: next_y,
        };

        current_tetrimino
    }

    fn apply_initial_displacement(tetrimino_type: &TetriminoType, x: isize, y: isize) -> (isize, isize) {
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

    fn next_window_xy(&self) -> (u16, u16) {
        (self.x + LEFT_PANEL_WIDTH + BOARD_WIDTH * 2 + 3, self.y)
    }

    fn held_window_xy(&self) -> (u16, u16) {
        (self.x + LEFT_PANEL_WIDTH + BOARD_WIDTH * 2 + 3, self.y + NEXT_WINDOW_HEIGHT + 1)
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

#[derive(Copy, Clone)]
enum TetriminoType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl TetriminoType {
    pub fn get_color(&self) -> Rgb {
        match self {
            TetriminoType::I => Rgb(0, 255, 255),
            TetriminoType::O => Rgb(255, 255, 0),
            TetriminoType::T => Rgb(128, 0, 128),
            TetriminoType::S => Rgb(0, 128, 0),
            TetriminoType::Z => Rgb(255, 0, 0),
            TetriminoType::J => Rgb(0, 0, 255),
            TetriminoType::L => Rgb(255, 165, 0),
        }
    }

    pub fn get_block(&self, state: usize) -> &[[u8; 4]; 4] {
        match self {
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
                ].get(state).unwrap()
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
                ].get(state).unwrap()
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
                ].get(state).unwrap()
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
                ].get(state).unwrap()
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
                ].get(state).unwrap()
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
                ].get(state).unwrap()
            }
        }
    }
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
        self.tetrimino_type.get_color()
    }

    pub fn to_block(&self) -> &[[u8; 4]; 4] {
        self.tetrimino_type.get_block(self.state)
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