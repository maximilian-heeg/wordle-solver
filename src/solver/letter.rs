use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Status {
    Unknown,
    Absent,
    Misplaced,
    Correct,
}

#[derive(Copy, Clone, Debug)]
pub struct Letter {
    pub letter: Option<char>,
    pub status: Status,
}

impl Default for Letter {
    fn default() -> Self {
        Self::new()
    }
}

impl Letter {
    pub fn new() -> Self {
        Letter {
            letter: None,
            status: Status::Unknown,
        }
    }

    pub fn set(&mut self, letter: Option<char>) {
        match letter {
            Some(letter) => {
                if letter.is_ascii_alphabetic() & letter.is_ascii_lowercase() {
                    self.letter = Some(letter);
                }
            }
            None => self.letter = None,
        }
    }

    pub fn render(self, area: Rect, buf: &mut Buffer, selected: bool) {
        let block = match selected {
            true => Block::new()
                .borders(Borders::ALL)
                .border_set(border::DOUBLE),
            false => Block::new().borders(Borders::ALL),
        };

        let style = match self.status {
            Status::Unknown => Style::default().bg(Color::Black),
            Status::Absent => Style::default().bg(Color::Red),
            Status::Misplaced => Style::default().bg(Color::LightYellow).fg(Color::Black),
            Status::Correct => Style::default()
                .bg(Color::LightGreen)
                .fg(Color::Black)
                .bold(),
        };

        let letter = match self.letter {
            Some(l) => l.to_uppercase(),
            _ => ' '.to_uppercase(),
        };
        Paragraph::new(letter.to_string())
            .bold()
            .centered()
            .block(block)
            .style(style)
            .render(area, buf);
    }
}
