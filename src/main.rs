use std::io;

use solver::word::Word;

mod app;
mod solver;
mod tui;

fn _create_word_from_string(word: &str) -> Word {
    let mut res = Word::new();
    for (i, letter) in word.chars().enumerate() {
        res.set_letter(Some(letter), i);
    }
    res
}

fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;
    let app_result = app::App::default().run(&mut terminal);
    tui::restore()?;
    app_result
}
