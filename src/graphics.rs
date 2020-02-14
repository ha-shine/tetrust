use std::io::{Result, Write};

use termion::cursor;

const TOP_LEFT_CORNER: &'static str = "╔";
const TOP_RIGHT_CORNER: &'static str = "╗";
const BOTTOM_LEFT_CORNER: &'static str = "╚";
const BOTTOM_RIGHT_CORNER: &'static str = "╝";
const VERTICAL_WALL: &'static str = "║";
const HORIZONTAL_WALL: &'static str = "═";

pub fn create_window<W>(w: &mut W, x: u16, y: u16, width: u16, height: u16) -> Result<()>
    where W: Write
{
    write!(w, "{}{}", cursor::Goto(x, y), TOP_LEFT_CORNER)?;
    write!(w, "{}{}", cursor::Goto(x + width - 1, y), TOP_RIGHT_CORNER)?;
    write!(w, "{}{}", cursor::Goto(x, y + height - 1), BOTTOM_LEFT_CORNER)?;
    write!(w, "{}{}", cursor::Goto(x + width - 1, y + height - 1), BOTTOM_RIGHT_CORNER)?;

    for i in 1..width - 1 {
        write!(w, "{}{}", cursor::Goto(x + i as u16, y), HORIZONTAL_WALL)?;
        write!(w, "{}{}", cursor::Goto(x + i as u16, y + height - 1), HORIZONTAL_WALL)?;
    }

    for i in 1..height - 1 {
        write!(w, "{}{}", cursor::Goto(x, y + i as u16), VERTICAL_WALL)?;
        write!(w, "{}{}", cursor::Goto(x + width - 1, y + i as u16), VERTICAL_WALL)?;
    }

    Ok(())
}