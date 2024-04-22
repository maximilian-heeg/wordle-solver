use std::iter::zip;
use tokio::runtime::Handle;

use super::{App, N_SUGGESTIONS};
use crate::wordlebot::wordle::{Guess, LetterStatus};
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};
use wordlebot::wordle::{decode_status, encode_status};

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border = self.create_border();

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(35), Constraint::Min(5)])
            .split(border.inner(area));

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(55), Constraint::Fill(1)])
            .split(rows[0]);

        self.render_guess_area(columns[0], buf);
        self.render_solver_area(columns[1], buf);
        self.render_chart(rows[1], buf);

        border.render(area, buf);
    }
}

// ANCHOR: centered_rect
/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(x: u16, y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(y),
            Constraint::Length(10),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(x),
            Constraint::Fill(1),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

impl App {
    fn create_border(&self) -> Block<'_> {
        let title = Title::from(" Wordlebot ".bold());
        let instructions = Title::from(Line::from(vec![
            " Quit ".into(),
            "<Esc> ".blue().bold(),
            " Toggle status ".into(),
            "<Tab> ".blue().bold(),
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
        block
    }

    fn render_guess_area(&self, area: Rect, buf: &mut Buffer) {
        // Render title
        let title = Title::from(" Your guesses ".bold());
        let block = Block::new()
            .title(title.alignment(Alignment::Center))
            .padding(Padding {
                left: 0,
                right: 0,
                top: 1,
                bottom: 0,
            });

        // Create two rows
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(N_SUGGESTIONS as u16 + 3),
                Constraint::Length(10),
                Constraint::Fill(1),
            ])
            .split(block.inner(area));

        self.render_evaluation(rows[1], buf);

        // Create the guess area
        let word_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3); 6])
            .split(rows[0]);
        for i in 0..6 {
            let selected_letter = match i {
                _ if i == self.selected_word => Some(self.selected_letter),
                _ => None,
            };
            let valid = self.solver.is_valid_guess(&self.cached_guesses[i].word);
            self.guesses[i].render(word_rows[i], buf, selected_letter, valid)
        }
        block.render(area, buf);
    }

    fn render_solver_area(&self, area: Rect, buf: &mut Buffer) {
        let title = Title::from("Solver".bold());
        let block = Block::new().title(title.alignment(Alignment::Center));

        // Create two rows
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(N_SUGGESTIONS as u16 + 4),
                Constraint::Fill(1),
            ])
            .split(block.inner(area));

        self.render_suggestions(rows[0], buf);

        // Plot all solutions
        let mut lines: Vec<Line<'_>> = vec![Line::from(vec![
            "Remaining words: ".bold(),
            self.remaining_words.len().to_string().bold().magenta(),
        ])];
        let solutions = self.solver.get_words_from_idx(&self.remaining_words);
        for item in solutions {
            lines.push(format!("{}", item).into())
        }
        Paragraph::new(lines)
            // .scroll((self.scroll, 0))
            .render(rows[1], buf);

        block.render(area, buf);
    }

    fn render_chart(&self, area: Rect, buf: &mut Buffer) {
        let i = if self.selected_word > self.evaludations.len() - 1 {
            self.evaludations.len() - 1
        } else {
            self.selected_word
        };
        if let Some(eval) = self.evaludations.get(i) {
            let status = match eval.status {
                Some(x) => encode_status(&x),
                None => 0,
            };
            let sizes: Vec<_> = eval
                .group_sizes
                .iter()
                .map(|(s, size)| {
                    let style = if s == &status {
                        Style::new().red()
                    } else {
                        Style::new().dark_gray()
                    };
                    Bar::default()
                        .value(*size as u64)
                        .style(style)
                        .text_value("".to_string())
                })
                .collect();

            let width = match area.width / sizes.len() as u16 {
                0 => 1,
                x if x > 10 => 10,
                x => x,
            };

            let chart = BarChart::default()
                .block(
                    Block::default()
                        .title(
                            Title::from(
                                format!(
                                    " Histogram of group sizes of guess number {}: {} ",
                                    i + 1,
                                    eval.word
                                )
                                .bold(),
                            )
                            .alignment(Alignment::Center),
                        )
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::new().dark_gray()),
                )
                .bar_width(width)
                .bar_gap(0)
                .data(BarGroup::default().bars(&sizes));
            chart.render(area, buf);
        }
    }

    fn render_evaluation(&self, area: Rect, buf: &mut Buffer) {
        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(51)])
            .flex(layout::Flex::Center)
            .split(area);

        let rows: Vec<_> = self
            .evaludations
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let style = if self.selected_word == i {
                    Style::new().white()
                } else {
                    Style::new()
                };
                Row::new(vec![
                    Text::from(format!("{}", w.word)).alignment(Alignment::Left),
                    Text::from(format!("{:.2}", w.expected_bits)).alignment(Alignment::Center),
                    Text::from(format!("{:.2}", w.real_bits.unwrap())).alignment(Alignment::Center),
                    Text::from(w.groups.to_string()).alignment(Alignment::Center),
                    Text::from(w.max_group_size.to_string()).alignment(Alignment::Center),
                    Text::from(w.n_remaining_after.unwrap().to_string())
                        .alignment(Alignment::Center),
                ])
                .style(style)
            })
            .collect();
        let widths = [
            Constraint::Length(5),
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Length(6),
            Constraint::Length(9),
            Constraint::Length(9),
        ];
        let table = Table::new(rows, widths)
            // ...and they can be separated by a fixed spacing.
            .column_spacing(1)
            // You can set the style of the entire Table.
            .style(Style::new())
            // It has an optional header, which is simply a Row always visible at the top.
            .header(
                Row::new(vec![
                    Cell::from("Guess").underlined(),
                    Cell::from("Exp. Bits").underlined(),
                    Cell::from("Act. Bits").underlined(),
                    Cell::from("groups").underlined(),
                    Cell::from("max group").underlined(),
                    Cell::from("remaining").underlined(),
                ])
                .style(Style::new()),
            )
            .block(
                Block::default()
                    .title(
                        Title::from("Evaluation of previous guesses").alignment(Alignment::Center),
                    )
                    .bold()
                    .padding(Padding::new(0, 0, 1, 0)),
            );
        ratatui::widgets::Widget::render(table, area[0], buf);
    }

    fn render_suggestions(&self, area: Rect, buf: &mut Buffer) {
        let two_level_style = if self.two_level { 7 } else { 0 };
        let rows: Vec<_> = self
            .suggestions
            .iter()
            .map(|w| {
                let style = if w.is_possible {
                    Style::default().white()
                } else {
                    Style::default()
                };

                let two_level_bits = w.two_level_bits.unwrap_or(0.);

                Row::new(vec![
                    Text::from(format!("{}", w.word))
                        .alignment(Alignment::Left)
                        .style(style),
                    Text::from(format!("{:.2}", w.expected_bits))
                        .alignment(Alignment::Center)
                        .style(style),
                    Text::from(format!("{:.2?}", two_level_bits))
                        .alignment(Alignment::Center)
                        .style(style),
                    Text::from(w.groups.to_string())
                        .alignment(Alignment::Center)
                        .style(style),
                    Text::from(w.max_group_size.to_string())
                        .alignment(Alignment::Center)
                        .style(style),
                    Text::from(format!("{:.2}", w.prior))
                        .alignment(Alignment::Center)
                        .style(style),
                ])
            })
            .collect();
        let widths = [
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(two_level_style),
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Length(5),
        ];
        let table = Table::new(rows, widths)
            // ...and they can be separated by a fixed spacing.
            .column_spacing(1)
            // You can set the style of the entire Table.
            .style(Style::new())
            // It has an optional header, which is simply a Row always visible at the top.
            .header(Row::new(vec![
                Cell::from("Suggestion").underlined(),
                Cell::from("Exp. Bits").underlined(),
                Cell::from("2-l Bits").underlined(),
                Cell::from("n groups").underlined(),
                Cell::from("max group").underlined(),
                Cell::from("prior").underlined(),
            ]))
            .block(Block::new().padding(Padding::new(0, 0, 1, 0)));
        ratatui::widgets::Widget::render(table, area, buf);

        // Check if active task
        let metrics = Handle::current().metrics();
        let n = metrics.active_tasks_count();
        if n > 1 {
            let popup_block = Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(Color::Red))
                .padding(Padding::uniform(1));
            let popup_area = centered_rect(30, 4, area);

            Clear.render(popup_area, buf);
            Paragraph::new(vec![
                Line::from("Working on the best"),
                Line::from("solutions for you"),
            ])
            .alignment(Alignment::Center)
            .white()
            .block(popup_block)
            .render(popup_area, buf);
        }
    }
}

trait RenderGuess {
    fn render(&self, area: Rect, buf: &mut Buffer, selected_letter: Option<usize>, valid: bool);
}

impl RenderGuess for Guess {
    fn render(&self, area: Rect, buf: &mut Buffer, selected_letter: Option<usize>, valid: bool) {
        let row_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Length(7); 5])
            .flex(layout::Flex::Center)
            .split(area);
        let decoded_status = decode_status(self.status);
        for (i, (letter, status)) in zip(self.word.chars, decoded_status).enumerate() {
            let border_style = if valid {
                match status {
                    LetterStatus::Absent => Style::default().white(),
                    LetterStatus::Misplaced => Style::default().light_yellow(),
                    LetterStatus::Correct => Style::default().light_green(),
                }
            } else {
                Style::default().dark_gray()
            };

            let text_style = if valid {
                match status {
                    LetterStatus::Absent => Style::default().bg(Color::Black),
                    LetterStatus::Misplaced => Style::default().fg(Color::LightYellow),
                    LetterStatus::Correct => Style::default().fg(Color::LightGreen).bold(),
                }
            } else {
                Style::default().dark_gray()
            };

            let block = match selected_letter {
                Some(pos) if i == pos => Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(border_style),
                // .border_style(Style::new().magenta()),
                _ => Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_style),
            };

            let letter = match letter {
                Some(l) => Text::styled(l.to_uppercase().to_string(), text_style),
                _ => Text::styled("".to_string(), text_style),
            };
            Paragraph::new(letter)
                .bold()
                .centered()
                .block(block)
                // .style(style)
                .render(row_layout[i], buf);
        }
    }
}
