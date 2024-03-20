use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};
use std::io;

mod solver;
use self::solver::Solver;

use super::tui;

mod word;
use word::*;

mod letter;
use letter::Status;

pub struct App {
    exit: bool,
    words: Vec<Word>,
    selected_word: usize,
    selected_letter: usize,
    solver: Solver,
    scroll: u16,
}

impl App {
    pub fn default() -> Self {
        App {
            exit: false,
            words: vec![Word::new()],
            selected_word: 0,
            selected_letter: 0,
            solver: Solver::new(),
            scroll: 0,
        }
    }
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        self.solver.update_remaining_words(&self.words);
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.exit(),
            KeyCode::Char('1') => self.add_word(),
            KeyCode::Char('9') => self.pop_word(),
            KeyCode::Right => self.move_right(),
            KeyCode::Left => self.move_left(),
            KeyCode::Down => self.move_down(),
            KeyCode::Up => self.move_up(),
            KeyCode::Char(x) if x.is_ascii_alphabetic() => self.set_letter(Some(x)),
            KeyCode::Delete => self.set_letter(None),
            KeyCode::Tab => self.toggle_status(),
            KeyCode::PageUp => self.scroll_up(),
            KeyCode::PageDown => self.scroll_down(),
            _ => {}
        }
    }

    fn scroll_up(&mut self) {
        if self.scroll <= 10 {
            self.scroll = 0
        } else {
            self.scroll -= 10
        }
    }

    fn scroll_down(&mut self) {
        self.scroll += 10;

        if self.scroll > self.solver.get_n_remaining_words() as u16 {
            self.scroll = self.solver.get_n_remaining_words() as u16
        }
    }

    /// Toggle the status of the current letter
    /// if the letter is set
    fn toggle_status(&mut self) {
        if self.words[self.selected_word].letters[self.selected_letter]
            .letter
            .is_some()
        {
            use Status::*;
            self.words[self.selected_word].letters[self.selected_letter].status =
                match self.words[self.selected_word].letters[self.selected_letter].status {
                    Unknown => Absent,
                    Absent => Misplaced,
                    Misplaced => Correct,
                    Correct => Unknown,
                };
        }
        self.solver.update_remaining_words(&self.words)
    }

    fn set_letter(&mut self, letter: Option<char>) {
        self.words[self.selected_word].set_letter(letter, self.selected_letter);
        if letter.is_none() {
            self.words[self.selected_word].letters[self.selected_letter].status = Status::Unknown;
            self.move_left()
        } else {
            self.move_right()
        }
        self.solver.update_remaining_words(&self.words)
    }

    fn move_right(&mut self) {
        if self.selected_letter < 4 {
            self.selected_letter += 1;
        }
    }

    fn move_left(&mut self) {
        if self.selected_letter > 0 {
            self.selected_letter -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.selected_word < self.words.len() - 1 {
            self.selected_word += 1;
        }
    }

    fn move_up(&mut self) {
        if self.selected_word > 0 {
            self.selected_word -= 1;
        }
    }

    fn pop_word(&mut self) {
        if self.words.len() > 1 {
            self.words.pop();
            if self.selected_word > self.words.len() - 1 {
                self.selected_word = self.words.len() - 1;
            }
        }
        self.solver.update_remaining_words(&self.words)
    }

    fn add_word(&mut self) {
        let word = Word::new();
        self.words.push(word);
        self.selected_letter = 0;
        self.selected_word = self.words.len() - 1;
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Worlde sovler ".bold());
        let instructions = Title::from(Line::from(vec![
            " Quit ".into(),
            "<Esc> ".blue().bold(),
            "Add word ".into(),
            "<1> ".blue().bold(),
            "Remove word ".into(),
            "<9> ".blue().bold(),
            "Toggle status ".into(),
            "<Tab> ".blue().bold(),
            "Scroll down ".into(),
            "<PageDown> ".blue().bold(),
            "Scroll up ".into(),
            "<PageUp> ".blue().bold(),
        ]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(symbols::border::PLAIN);
        block.render(area, buf);

        let n_words = self.words.len();

        let columns = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(2)
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(45), Constraint::Fill(1)])
            .split(area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3); n_words])
            .split(columns[0]);

        for i in 0..n_words {
            let selected_letter = match i {
                _ if i == self.selected_word => Some(self.selected_letter),
                _ => None,
            };
            self.words[i].render(layout[i], buf, selected_letter)
        }

        // Plot Header
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Fill(1)])
            .split(columns[1]);

        // next guess
        let next_guess_frequency = self
            .solver
            .next_guess_frequency(3)
            .into_iter()
            .map(|g| format!("{g}"))
            .collect::<Vec<String>>()
            .join(", ");
        let next_guess_groups = self
            .solver
            .next_guess_groups(3)
            .into_iter()
            .map(|g| format!("{g}"))
            .collect::<Vec<String>>()
            .join(", ");
        let next_guess_combined = self
            .solver
            .next_guess(3, self.words.len())
            .into_iter()
            .map(|g| format!("{g}"))
            .collect::<Vec<String>>()
            .join(", ");
        Paragraph::new(vec![
            Line::from(vec![
                "Possible solutions: ".bold().blue(),
                self.solver.get_n_remaining_words().to_string().into(),
            ]),
            Line::from(vec![
                "Best next guesses (Freq.): ".bold().blue(),
                next_guess_frequency.into(),
            ]),
            Line::from(vec![
                "Best next guesses (Groups): ".bold().blue(),
                next_guess_groups.into(),
            ]),
            Line::from(vec![
                "Best next guesses (Combined): ".bold().blue(),
                next_guess_combined.into(),
            ]),
        ])
        .render(rows[0], buf);

        // Plot all solutions
        let mut lines: Vec<Line<'_>> = vec![];
        let solutions = self.solver.get_remaining_words();
        for item in solutions {
            lines.push(format!("{}", item).into())
        }
        Paragraph::new(lines)
            .scroll((self.scroll, 0))
            .render(rows[1], buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_key_event() -> io::Result<()> {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Esc.into());
        assert_eq!(app.exit, true);

        Ok(())
    }
}
