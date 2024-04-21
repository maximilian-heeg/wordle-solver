use super::*;
use crossterm::event::{KeyCode, KeyEvent};

impl App {
    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.exit(),

            // Navigation
            KeyCode::Right => self.move_right(),
            KeyCode::Left => self.move_left(),
            KeyCode::Down => self.move_down(),
            KeyCode::Up => self.move_up(),
            KeyCode::Enter => {
                self.move_down();
                self.selected_letter = 0;
            }

            // Enter words
            KeyCode::Char(x) if x.is_ascii_alphabetic() => {
                self.set_letter(Some(x));
                self.move_right()
            }
            KeyCode::Backspace => {
                self.set_letter(None);
                self.move_left()
            }
            KeyCode::Tab => self.toggle_status(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
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
        if self.selected_word < self.guesses.len() - 1 {
            self.selected_word += 1;
        }
    }

    fn move_up(&mut self) {
        if self.selected_word > 0 {
            self.selected_word -= 1;
        }
    }

    fn set_letter(&mut self, letter: Option<char>) {
        self.guesses[self.selected_word].set_letter(letter, self.selected_letter);
        if letter.is_none() {
            self.guesses[self.selected_word]
                .update_status(LetterStatus::Absent, self.selected_letter)
        }
        self.update_guesses();
    }

    fn toggle_status(&mut self) {
        if self.guesses[self.selected_word].word.chars[self.selected_letter].is_some() {
            use LetterStatus::*;
            let current =
                decode_status(self.guesses[self.selected_word].status)[self.selected_letter];
            let new = match current {
                Absent => Misplaced,
                Misplaced => Correct,
                Correct => Absent,
            };
            self.guesses[self.selected_word].update_status(new, self.selected_letter);
            self.update_guesses();
        }
    }
}
