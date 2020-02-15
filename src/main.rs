use std::io::stdout;
use tetrust::game::Game;
use termion::async_stdin;

fn main() {
    let mut game = Game::new(1, 1, async_stdin(), stdout());
    game.start().unwrap();
}