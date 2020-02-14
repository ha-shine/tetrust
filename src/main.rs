use std::io::stdout;
use tetrust::game::Game;

fn main() {
    let mut game = Game::new(1, 1, stdout());
    game.start().unwrap();
}