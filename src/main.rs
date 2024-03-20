use std::io;

mod app;
mod tui;

fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;
    let app_result = app::App::default().run(&mut terminal);
    tui::restore()?;
    app_result
}
