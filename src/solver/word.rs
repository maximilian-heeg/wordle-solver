use super::letter::*;
use ratatui::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Word {
    pub letters: [Letter; 5],
}

impl Default for Word {
    fn default() -> Self {
        Self::new()
    }
}

impl Word {
    pub fn new() -> Self {
        let letter = Letter::new();
        Word {
            letters: [letter; 5],
        }
    }

    pub fn set_letter(&mut self, letter: Option<char>, position: usize) {
        self.letters[position].set(letter);
    }

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

    fn has_letter_at_position(&self, test: char, pos: usize) -> bool {
        match self.letters[pos].letter {
            Some(c) => c == test,
            None => false,
        }
    }

    fn count_letter(&self, letter: &char) -> usize {
        self.letters
            .iter()
            .filter(|l| match l.letter {
                Some(c) => &c == letter,
                None => false,
            })
            .count()
    }

    fn filter_letters(&self) -> Word {
        let mut word = *self;
        for i in 0..word.letters.len() {
            if word.letters[i].status == Status::Unknown {
                word.letters[i].letter = None;
            } else if word.letters[i].status == Status::Absent {
                word.letters[i].letter = None
            }
        }
        word
    }

    pub fn compare(&self, guess: &Word) -> [Status; 5] {
        let mut result = [Status::Absent; 5];
        let mut remaining: Vec<usize> = vec![];

        // Find all correct letters
        guess
            .letters
            .iter()
            .enumerate()
            .for_each(|(i, guess_char)| {
                if Some(guess_char.letter) == Some(self.letters[i].letter) {
                    result[i] = Status::Correct;
                } else {
                    remaining.push(i);
                }
            });

        // Loop though remeining
        let mut word = *self;
        for pos in remaining.iter() {
            let guess_letter = guess.letters[*pos].letter;
            for word_pos in remaining.iter() {
                if guess_letter == word.letters[*word_pos].letter {
                    result[*pos] = Status::Misplaced;
                    word.letters[*word_pos].letter = None;
                    break;
                }
            }
        }

        result
    }

    pub fn is_valid(&self, guess: &Word) -> bool {
        // let must_letters = guess.filter_letters().count_letters();
        // let word_letters: HashMap<char, usize> = self.count_letters();

        for (guess_pos, guess_letter) in guess.letters.iter().enumerate() {
            if let Some(guess_char) = guess_letter.letter {
                match guess_letter.status {
                    Status::Unknown => (),
                    Status::Absent => match guess.filter_letters().count_letter(&guess_char) {
                        // The letter must appear somewhere, but not at this position
                        n_must if n_must > 0 => {
                            if self.has_letter_at_position(guess_char, guess_pos) {
                                return false;
                            };
                            let n_is = self.count_letter(&guess_char);
                            // println!("Incor: {n_must} {n_is}");
                            if n_is > n_must {
                                return false;
                            };
                        }
                        // The letter must not appear at all
                        _ => {
                            if self.count_letter(&guess_char) > 0 {
                                return false;
                            }
                        }
                    },
                    Status::Misplaced => {
                        if self.has_letter_at_position(guess_char, guess_pos) {
                            return false;
                        }
                        let n_must = guess.filter_letters().count_letter(&guess_char);
                        let n_is = self.count_letter(&guess_char);
                        if n_is == 0 || n_must > n_is {
                            return false;
                        }
                    }
                    Status::Correct => {
                        if !self.has_letter_at_position(guess_char, guess_pos) {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for letter in self.letters.iter() {
            if let Some(c) = letter.letter {
                write!(f, "{}", c)?;
            } else {
                write!(f, "_")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    fn create_word_from_string(word: &str) -> Word {
        let mut res = Word::new();
        for (i, letter) in word.chars().enumerate() {
            res.set_letter(Some(letter), i);
        }
        res
    }

    #[test]
    fn has_letter_at_position() -> io::Result<()> {
        let word = create_word_from_string("slate");
        assert_eq!(word.has_letter_at_position('l', 1), true);
        assert_eq!(word.has_letter_at_position('l', 2), false);
        Ok(())
    }

    #[test]
    fn count_letter() -> io::Result<()> {
        let word = create_word_from_string("goose");

        assert_eq!(word.count_letter(&'g'), 1);
        assert_eq!(word.count_letter(&'o'), 2);
        assert_eq!(word.count_letter(&'s'), 1);
        assert_eq!(word.count_letter(&'e'), 1);
        Ok(())
    }

    #[test]
    fn count_letters_filter() -> io::Result<()> {
        let mut word = create_word_from_string("goose");
        word.letters[1].status = Status::Misplaced;
        word.letters[2].status = Status::Correct;
        let word = word.filter_letters();
        assert_eq!(word.count_letter(&'o'), 2);
        Ok(())
    }

    #[test]
    fn keep_word_correct_letter() -> io::Result<()> {
        let mut guess = create_word_from_string("slate");
        guess.letters[2].status = Status::Correct;
        assert!(create_word_from_string("plate").is_valid(&guess));
        assert!(!create_word_from_string("water").is_valid(&guess));
        Ok(())
    }

    #[test]
    fn keep_word_incorrect_letter() -> io::Result<()> {
        let mut guess = create_word_from_string("slate");
        guess.letters[2].status = Status::Absent;
        assert!(!create_word_from_string("plate").is_valid(&guess));
        assert!(!create_word_from_string("water").is_valid(&guess));
        assert!(create_word_from_string("songs").is_valid(&guess));
        Ok(())
    }

    #[test]
    fn keep_word_incorrect_letter_2() -> io::Result<()> {
        let mut guess = create_word_from_string("goose");
        guess.letters[2].status = Status::Absent;
        guess.letters[1].status = Status::Misplaced;
        assert!(create_word_from_string("bacon").is_valid(&guess));
        assert!(!create_word_from_string("bloom").is_valid(&guess));
        Ok(())
    }

    #[test]
    fn keep_word_misplaced_letter() -> io::Result<()> {
        let mut guess = create_word_from_string("bacon");
        guess.letters[4].status = Status::Correct;
        guess.letters[3].status = Status::Misplaced;
        assert!(create_word_from_string("ronin").is_valid(&guess));
        assert!(!create_word_from_string("resin").is_valid(&guess));
        Ok(())
    }

    #[test]
    fn keep_word_misplaced_letter_2() -> io::Result<()> {
        let mut guess = create_word_from_string("feels");
        guess.letters[1].status = Status::Misplaced;
        guess.letters[2].status = Status::Misplaced;
        assert!(create_word_from_string("agree").is_valid(&guess));
        assert!(!create_word_from_string("resin").is_valid(&guess));
        assert!(!create_word_from_string("brake").is_valid(&guess));
        Ok(())
    }

    #[test]
    fn keep_word_misplaced_letter_3() -> io::Result<()> {
        let mut guess = create_word_from_string("esses");
        guess.letters[0].status = Status::Misplaced;
        guess.letters[1].status = Status::Absent;
        guess.letters[2].status = Status::Absent;
        guess.letters[3].status = Status::Absent;
        guess.letters[4].status = Status::Absent;
        assert!(!create_word_from_string("reede").is_valid(&guess));
        Ok(())
    }

    #[test]
    fn compare_words() -> io::Result<()> {
        let word = create_word_from_string("water");

        let guess = create_word_from_string("slate");
        let expected = [
            Status::Absent,
            Status::Absent,
            Status::Misplaced,
            Status::Misplaced,
            Status::Misplaced,
        ];
        assert_eq!(word.compare(&guess), expected);

        let guess = create_word_from_string("eerie");
        let expected = [
            Status::Misplaced,
            Status::Absent,
            Status::Misplaced,
            Status::Absent,
            Status::Absent,
        ];
        assert_eq!(word.compare(&guess), expected);

        let guess = create_word_from_string("eater");
        let expected = [
            Status::Absent,
            Status::Correct,
            Status::Correct,
            Status::Correct,
            Status::Correct,
        ];
        assert_eq!(word.compare(&guess), expected);

        Ok(())
    }

    #[test]
    fn compare_words_2() -> io::Result<()> {
        let word = create_word_from_string("steer");

        let guess = create_word_from_string("slate");
        let expected = [
            Status::Correct,
            Status::Absent,
            Status::Absent,
            Status::Misplaced,
            Status::Misplaced,
        ];
        assert_eq!(word.compare(&guess), expected);

        let guess = create_word_from_string("deers");
        let expected = [
            Status::Absent,
            Status::Misplaced,
            Status::Correct,
            Status::Misplaced,
            Status::Misplaced,
        ];
        assert_eq!(word.compare(&guess), expected);

        Ok(())
    }
}
