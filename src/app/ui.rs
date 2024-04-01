use crate::solver::letter;
use crate::solver::letter::Status;
use crate::solver::word;

use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};

impl word::Word {
    pub fn render(&self, area: Rect, buf: &mut Buffer, selected_letter: Option<usize>) {
        let row_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(7); 5])
            .flex(layout::Flex::Center)
            .split(area);
        for i in 0..5 {
            let selected = match selected_letter {
                // Check if the current position needs to be highlighted
                Some(position) if position == i => true,
                // All other cases
                _ => false,
            };
            self.letters[i].render(row_layout[i], buf, selected)
        }
    }
}

impl letter::Letter {
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
