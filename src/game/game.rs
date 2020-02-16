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
use rand::rngs::ThreadRng;
use rand::thread_rng;
use rand::seq::SliceRandom;
use crate::game::tetrimino::{Tetrimino, Type};

const LEFT_PANEL_WIDTH: u16 = 17;

const SCORE_WINDOW_HEIGHT: u16 = 8;
const HELP_WINDOW_HEIGHT: u16 = 13;

const BOARD_WIDTH: u16 = 10;
const BOARD_HEIGHT: u16 = 20;

const RIGHT_PANEL_WIDTH: u16 = 12;
const NEXT_WINDOW_HEIGHT: u16 = 10;
const HELD_WINDOW_HEIGHT: u16 = 10;

pub struct Game<R: Read, W: Write> {
    x: u16,
    y: u16,
    score: usize,
    lines: usize,
    board: Board,
    stdin: Keys<R>,
    stdout: W,
    state: State,
    current_tetrimino: ActiveTetrimino,
    next_type: Type,
    held_type: Option<Type>,
    can_hold: bool,
    elapsed: Duration,
    fall_rate: Duration,
    generator: SevenGenerator,
}

enum State {
    Playing,
    Lost,
}

struct ActiveTetrimino {
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

impl<R: Read, W: Write> Game<R, W> {
    pub fn new(x: u16, y: u16, r: R, w: W) -> Game<R, RawTerminal<W>> {
        let mut generator = SevenGenerator::new();
        let current_ttype = generator.next().unwrap();
        let next_ttype = generator.next().unwrap();

        Game {
            x,
            y,
            score: 0,
            lines: 0,
            board: Board::new(),
            stdin: r.keys(),
            stdout: w.into_raw_mode().unwrap(),
            state: State::Playing,
            current_tetrimino: Self::initialize_tetrimino(current_ttype),
            next_type: next_ttype,
            held_type: None,
            can_hold: true,
            elapsed: Duration::from_millis(0),
            fall_rate: Duration::from_millis(500),
            generator,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        write!(&mut self.stdout, "{}{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1), cursor::Hide)?;

        'main: loop {
            if let State::Lost = self.state {
                thread::sleep(Duration::from_millis(50));
                if let Some(Ok(Key::Char('q'))) = self.stdin.next() {
                    break 'main;
                }
                continue;
            }

            thread::sleep(Duration::from_millis(50));
            self.update(Duration::from_millis(50));

            match self.stdin.next() {
                Some(Ok(key)) => {
                    match key {
                        Key::Char('j') | Key::Left => self.handle_tetrimino_move(-1, 0),
                        Key::Char('l') | Key::Right => self.handle_tetrimino_move(1, 0),
                        Key::Char('k') | Key::Down => self.handle_tetrimino_move(0, 1),
                        Key::Char('x') => {
                            let new_state = self.current_tetrimino.tetrimino.rotate_clockwise();
                            if self.can_fit_tetrimino(self.current_tetrimino.x, self.current_tetrimino.y, new_state.block()) {
                                self.current_tetrimino.tetrimino = new_state;
                            }
                        }
                        Key::Char('z') => {
                            let new_state = self.current_tetrimino.tetrimino.rotate_counter_clockwise();
                            if self.can_fit_tetrimino(self.current_tetrimino.x, self.current_tetrimino.y, new_state.block()) {
                                self.current_tetrimino.tetrimino = new_state;
                            }
                        }
                        Key::Char(' ') => self.drop_current_tetrimino(),
                        Key::Char('c') => self.try_hold_tetrimino(),
                        Key::Char('q') => break 'main,
                        _ => {}
                    }
                }
                _ => {}
            }

            self.try_fuse_with_ground();
            self.erase_lines();
            self.check_game_over();
            self.draw_player_score()?;
            self.draw_help()?;
            self.draw_board()?;
            self.draw_next()?;
            self.draw_held()?;
            self.stdout.flush()?;
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

    fn check_game_over(&mut self) {
        for col in &self.board.blocks[0] {
            if let Block::Occupied(_) = col {
                self.state = State::Lost;
                break;
            }
        }
    }

    fn drop_current_tetrimino(&mut self) {
        for _ in 0..BOARD_HEIGHT {
            self.handle_tetrimino_move(0, 1);
        }
    }

    fn try_hold_tetrimino(&mut self) {
        if !self.can_hold {
            return;
        }

        if let Some(current) = self.held_type.take() {
            self.held_type = Some(self.current_tetrimino.tetrimino.ttype);
            self.current_tetrimino = Self::initialize_tetrimino(current);
        } else {
            self.held_type = Some(self.current_tetrimino.tetrimino.ttype);
            self.current_tetrimino = Self::initialize_tetrimino(self.next_type);
            self.next_type = self.generator.next().unwrap();
        }

        self.can_hold = false;
    }

    fn handle_tetrimino_move(&mut self, dx: isize, dy: isize) {
        let new_x = self.current_tetrimino.x as isize + dx;
        let new_y = self.current_tetrimino.y as isize + dy;

        if self.can_fit_tetrimino(new_x, new_y, self.current_tetrimino.tetrimino.block()) {
            self.current_tetrimino.x = new_x;
            self.current_tetrimino.y = new_y;
        }
    }

    // check whether given tetrimino block can fit in if it starts at block_x, block_y
    // e.g if block_x = 4, and block_y = 5, block[0][0] is at the board[5][4]
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
        let tetrimino_block = self.current_tetrimino.tetrimino.block();

        if self.should_fuse_current_tetrimino() {
            for (y, row) in tetrimino_block.iter().enumerate() {
                for (x, col) in row.iter().enumerate() {
                    if *col == 1 {
                        let rgb = self.current_tetrimino.tetrimino.color();
                        let x = self.current_tetrimino.x + x as isize;
                        let y = self.current_tetrimino.y + y as isize;
                        self.board.blocks[y as usize][x as usize] = Block::Occupied(rgb);
                    }
                }
            }

            self.current_tetrimino = Self::initialize_tetrimino(self.next_type);
            self.next_type = self.generator.next().unwrap();
            self.can_hold = true;
        }
    }

    // check whether current active tetrimino should be fused with the ground
    fn should_fuse_current_tetrimino(&self) -> bool {
        for (y, row) in self.current_tetrimino.tetrimino.block().iter().enumerate() {
            for (x, square) in row.iter().enumerate() {
                // if current square is 1 (has block), check if we're at the edge or the block on the next row
                if *square == 1 {
                    let x_on_board = self.current_tetrimino.x + x as isize;
                    let y_on_board = self.current_tetrimino.y + y as isize;

                    if y_on_board == BOARD_HEIGHT as isize - 1 {
                        return true;
                    } else if let Block::Occupied(_) = self.board.blocks[y_on_board as usize + 1][x_on_board as usize] {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn erase_lines(&mut self) {
        let mut erasable_lines = Vec::new();

        // doesn't need to iterate through all the boards, can optimise later
        for (y, row) in self.board.blocks.iter().enumerate() {
            if Self::can_erase_row(row) {
                erasable_lines.push(y);
            }
        }

        self.lines += erasable_lines.len();
        self.score += erasable_lines.len() * 10;

        // push down the lines and erase the top line
        for line in erasable_lines {
            for y in (0..line).rev() {
                for x in 0..BOARD_WIDTH {
                    self.board.blocks[y as usize + 1][x as usize] = self.board.blocks[y as usize][x as usize];
                }
            }
            for x in 0..BOARD_WIDTH {
                self.board.blocks[0][x as usize] = Block::Free;
            }
        }
    }

    fn can_erase_row(row: &[Block; BOARD_WIDTH as usize]) -> bool {
        for col in row {
            if let Block::Free = col {
                return false;
            }
        }

        true
    }

    fn draw_player_score(&mut self) -> Result<()> {
        let (x, y) = (self.x, self.y);
        create_window(&mut self.stdout, x, y, LEFT_PANEL_WIDTH, SCORE_WINDOW_HEIGHT)?;
        write!(self.stdout, "{}Score", cursor::Goto(x + 6, y + 2))?;
        write!(self.stdout, "{}score: {:04} ", cursor::Goto(x + 3, y + 4), self.score)?;
        write!(self.stdout, "{}lines: {:04} ", cursor::Goto(x + 3, y + 5), self.lines)
    }

    fn draw_help(&mut self) -> Result<()> {
        let (x, y) = (self.x, self.y + SCORE_WINDOW_HEIGHT + 1);
        create_window(&mut self.stdout, x, y, LEFT_PANEL_WIDTH, HELP_WINDOW_HEIGHT)?;
        write!(self.stdout, "{}Ctrls", cursor::Goto(x + 6, y + 2))?;
        write!(self.stdout, "{}left   j, ←", cursor::Goto(x + 3, y + 4))?;
        write!(self.stdout, "{}right  l, →", cursor::Goto(x + 3, y + 5))?;
        write!(self.stdout, "{}down   k, ↓", cursor::Goto(x + 3, y + 6))?;
        write!(self.stdout, "{}drop   space", cursor::Goto(x + 3, y + 7))?;
        write!(self.stdout, "{}rotate x, z", cursor::Goto(x + 3, y + 8))?;
        write!(self.stdout, "{}hold   c", cursor::Goto(x + 3, y + 9))?;
        write!(self.stdout, "{}quit   q", cursor::Goto(x + 3, y + 10))
    }

    fn draw_board(&mut self) -> Result<()> {
        let (init_x, init_y) = (self.x + LEFT_PANEL_WIDTH + 1, self.y);
        create_window(&mut self.stdout, init_x, init_y, (BOARD_WIDTH * 2) + 2, BOARD_HEIGHT + 2)?;

        // draw the board
        let (init_x, init_y) = (init_x + 1, init_y + 1);
        for (y, row) in self.board.blocks.iter().enumerate() {
            for (x, col) in row.iter().enumerate() {
                match col {
                    Block::Free => write!(self.stdout, "{}{}  ",
                                          cursor::Goto(init_x + (x * 2) as u16, init_y + y as u16),
                                          style::Reset)?,
                    Block::Occupied(rgb) => write!(self.stdout, "{}{}  ",
                                                   cursor::Goto(init_x + (x * 2) as u16, init_y + y as u16),
                                                   Bg(*rgb))?
                }
            }
        }

        match self.state {
            State::Lost => Ok(()),

            // draw current tetrimino, doesn't need to care about bounds, we can't even move outside of bound!
            State::Playing => self.draw_tetrimino(init_x as isize + self.current_tetrimino.x * 2,
                                                  init_y as isize + self.current_tetrimino.y,
                                                  65535, 65535,
                                                  self.current_tetrimino.tetrimino.ttype,
                                                  self.current_tetrimino.tetrimino.state)
        }
    }

    fn draw_next(&mut self) -> Result<()> {
        let (x, y) = (self.x + LEFT_PANEL_WIDTH + BOARD_WIDTH * 2 + 4, self.y);
        create_window(&mut self.stdout, x, y, RIGHT_PANEL_WIDTH, NEXT_WINDOW_HEIGHT)?;
        write!(self.stdout, "{}Next", cursor::Goto(x + 4, y + 2))?;

        for i in 0..4 {
            write!(self.stdout, "{}        ", cursor::Goto(x + 2, y + i + 4))?; // clear first
        }

        self.draw_tetrimino(x as isize + 2, y as isize + 4, x as isize + 9, y as isize + 9, self.next_type, 0)?;

        Ok(())
    }

    fn draw_held(&mut self) -> Result<()> {
        let (x, y) = (self.x + LEFT_PANEL_WIDTH + BOARD_WIDTH * 2 + 4, self.y + NEXT_WINDOW_HEIGHT + 2);
        create_window(&mut self.stdout, x, y, RIGHT_PANEL_WIDTH, HELD_WINDOW_HEIGHT)?;
        write!(self.stdout, "{}Held", cursor::Goto(x + 4, y + 2))?;

        for i in 0..4 {
            write!(self.stdout, "{}        ", cursor::Goto(x + 2, y + i + 4))?; // clear first
        }

        if let Some(held) = self.held_type {
            self.draw_tetrimino(x as isize + 2, y as isize + 4, x as isize + 9, y as isize + 9, held, 0)?;
        }

        Ok(())
    }

    fn draw_tetrimino(&mut self, x: isize, y: isize, bound_x: isize, bound_y: isize, ttype: Type, state: usize) -> Result<()> {
        let block = Tetrimino::block_of(ttype, state);
        let color = Tetrimino::color_of(ttype);

        for (yi, row) in block.iter().enumerate() {
            for (xi, col) in row.iter().enumerate() {
                let x = x + xi as isize * 2;
                let y = y + yi as isize;

                if *col == 1 && x >= 0 && x < bound_x && y >= 0 && y <= bound_y {
                    write!(self.stdout, "{}{}  {}", cursor::Goto(x as u16, y as u16), Bg(color), style::Reset)?;
                }
            }
        }

        write!(self.stdout, "{}", style::Reset)
    }

    fn initialize_tetrimino(ttype: Type) -> ActiveTetrimino {
        let (x, y) = match ttype {
            Type::I => (3, -1),
            _ => (3, 0)
        };

        let current_tetrimino = ActiveTetrimino {
            tetrimino: Tetrimino::new(ttype),
            x,
            y,
        };

        current_tetrimino
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

struct SevenGenerator {
    rng: ThreadRng,
    types: [Type; 7],
    idx: usize,
}

impl SevenGenerator {
    fn new() -> Self {
        use super::tetrimino::Type::*;

        let mut rng = thread_rng();
        let mut pieces = [I, O, T, S, Z, J, L];
        pieces.shuffle(&mut rng);

        SevenGenerator {
            rng,
            types: pieces,
            idx: 0,
        }
    }
}

impl Iterator for SevenGenerator {
    type Item = Type;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 6 {
            self.idx = 0;
            self.types.shuffle(&mut self.rng);
        }

        let current = self.types[self.idx];
        self.idx += 1;
        Some(current)
    }
}