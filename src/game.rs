use std::thread;
use std::io::{Result, Write};
use std::time::Duration;

use termion::{clear, cursor, style};
use termion::color::{Bg, Blue, Yellow};

use crate::graphics::*;

pub struct Game<W: Write> {
    x: u16,
    y: u16,
    score: u16,
    lines: u16,
    board: Board,
    stdout: W,
}

const LEFT_PANEL_WIDTH: u16 = 17;

const SCORE_WINDOW_HEIGHT: u16 = 8;
const HELP_WINDOW_HEIGHT: u16 = 12;

const BOARD_WIDTH: u16 = 10;
const BOARD_HEIGHT: u16 = 20;


impl<W: Write> Game<W> {
    pub fn new(x: u16, y: u16, w: W) -> Self {
        Game {
            x,
            y,
            score: 0,
            lines: 0,
            board: Board::new(),
            stdout: w,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        loop {
            write!(&mut self.stdout, "{}{}{}", clear::All, cursor::Goto(1, 1), cursor::Hide)?;

            self.draw_player_score()?;
            self.draw_help()?;
            self.draw_board()?;
            self.stdout.flush()?;

            thread::sleep(Duration::from_secs(1));
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

        for row in &self.board.blocks {
            let mut x = init_x + 1;
            for col in row {
                match col {
                    Block::Free => {
                        write!(self.stdout, "{}{}  ", cursor::Goto(x, y), Bg(Blue))?;
                    }
                    Block::Occupied => {
                        write!(self.stdout, "{}{}  ", cursor::Goto(x, y), Bg(Yellow))?;
                    }
                }
                x += 2;
            }
            y += 1;
        }

        write!(self.stdout, "{}", style::Reset)
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
    Occupied,
}

impl Board {
    fn new() -> Self {
        let blocks = [[Block::Free; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];
        Board { blocks }
    }
}