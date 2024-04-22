use std::fmt;

const NLETTER: usize = 5;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LetterStatus {
    Absent = 0,
    Misplaced = 1,
    Correct = 2,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Word {
    pub chars: [Option<char>; NLETTER],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Guess {
    pub word: Word,
    pub status: u8,
}

impl Default for Word {
    fn default() -> Self {
        Self::new()
    }
}

impl Word {
    /// Create a new word with empty letters
    pub fn new() -> Word {
        Word { chars: [None; 5] }
    }

    /// Set the letter at a position of the word
    ///
    /// # Example
    ///
    /// ```
    /// use wordlebot::wordle::Word;
    /// let mut word = Word::new();
    /// word.set_letter(Some('e'), 0);
    /// assert_eq!(word.chars[0], Some('e'));
    /// ```
    pub fn set_letter(&mut self, char: Option<char>, position: usize) {
        self.chars[position] = char
    }

    /// Compares the word to a guess, and returns the status code for the
    /// guess.
    ///
    /// # Example
    ///
    /// ```
    /// use wordlebot::wordle::*;
    /// use wordlebot::wordle::LetterStatus::*;
    /// let solution = create_word_from_string("tarse");
    /// let guess = create_word_from_string("slate");
    /// let expected = [Misplaced, Absent, Misplaced, Misplaced, Correct];
    /// assert_eq!(solution.compare(&guess), expected);
    ///
    /// ```
    pub fn compare(&self, guess: &Word) -> [LetterStatus; NLETTER] {
        let mut result = [LetterStatus::Absent; 5];
        let mut remaining_positions: Vec<usize> = vec![];

        // Find all correct letters
        guess
            .chars
            .iter()
            .enumerate()
            .for_each(|(i, guessed_char)| {
                if guessed_char == &self.chars[i] {
                    result[i] = LetterStatus::Correct;
                } else {
                    remaining_positions.push(i);
                }
            });

        // Loop though remeining
        let mut word = self.chars;
        for &pos in &remaining_positions {
            let guess_letter = guess.chars[pos];
            if let Some(&word_pos) = remaining_positions
                .iter()
                .find(|&word_pos| guess_letter == word[*word_pos])
            {
                result[pos] = LetterStatus::Misplaced;
                word[word_pos] = None;
            }
        }

        result
    }

    /// Counts the occrences of a char in a word
    ///
    /// # Example
    ///
    /// ```
    /// use wordlebot::wordle::*;
    ///
    /// let word = create_word_from_string("goose");
    /// assert_eq!(word.count_char(&'g'), 1);
    /// assert_eq!(word.count_char(&'t'), 0);
    /// assert_eq!(word.count_char(&'o'), 2);
    /// ```
    pub fn count_char(&self, char: &char) -> usize {
        self.chars
            .iter()
            .filter(|l| match l {
                Some(c) => c == char,
                None => false,
            })
            .count()
    }

    fn has_letter_at_position(&self, char: &char, position: usize) -> bool {
        match self.chars[position] {
            Some(c) => c == *char,
            None => false,
        }
    }

    /// Test if the current word is valid for a given guess.
    ///
    /// # Example
    ///
    /// ```
    /// use wordlebot::wordle::*;
    /// use wordlebot::wordle::LetterStatus::*;
    /// let guess = Guess::new("slate", [Correct, Absent, Absent, Absent, Absent]);
    /// assert!(!create_word_from_string("plate").is_valid(&guess));
    /// assert!(!create_word_from_string("water").is_valid(&guess));
    /// assert!(create_word_from_string("songs").is_valid(&guess));
    /// ```
    pub fn is_valid(&self, guess: &Guess) -> bool {
        let status = decode_status(guess.status);

        for (guess_pos, guess_letter) in guess.word.chars.iter().enumerate() {
            if let Some(guess_char) = guess_letter {
                match status[guess_pos] {
                    LetterStatus::Absent => {
                        match guess.remove_absent().count_char(guess_char) {
                            // The letter must appear somewhere, but not at this position
                            n_must if n_must > 0 => {
                                if self.has_letter_at_position(guess_char, guess_pos) {
                                    return false;
                                };
                                let n_is = self.count_char(guess_char);
                                // println!("Incor: {n_must} {n_is}");
                                if n_is > n_must {
                                    return false;
                                };
                            }
                            // The letter must not appear at all
                            _ => {
                                if self.count_char(guess_char) > 0 {
                                    return false;
                                }
                            }
                        }
                    }
                    LetterStatus::Misplaced => {
                        if self.has_letter_at_position(guess_char, guess_pos) {
                            return false;
                        }
                        let n_must = guess.remove_absent().count_char(guess_char);
                        let n_is = self.count_char(guess_char);
                        if n_is == 0 || n_must > n_is {
                            return false;
                        }
                    }
                    LetterStatus::Correct => {
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

impl Guess {
    /// Create a new guess from a string
    pub fn new(word: &str, status: [LetterStatus; 5]) -> Guess {
        let word = create_word_from_string(word);
        let status = encode_status(&status);
        Guess { word, status }
    }

    pub fn empty() -> Guess {
        Guess {
            word: Word::new(),
            status: 0,
        }
    }

    pub fn from_word(word: Word, status: [LetterStatus; 5]) -> Guess {
        let status = encode_status(&status);
        Guess { word, status }
    }

    // Reexport function from word
    pub fn set_letter(&mut self, char: Option<char>, position: usize) {
        self.word.set_letter(char, position)
    }

    pub fn set_status(&mut self, status: &[LetterStatus; NLETTER]) {
        self.status = encode_status(status)
    }

    pub fn get_status(&self) -> [LetterStatus; 5] {
        decode_status(self.status)
    }

    pub fn update_status(&mut self, status: LetterStatus, position: usize) {
        let mut current = self.get_status();
        current[position] = status;
        self.set_status(&current);
    }

    pub fn count_char(&self, char: &char) -> usize {
        self.word.count_char(char)
    }

    fn remove_absent(&self) -> Word {
        let mut word = self.word;
        let status = decode_status(self.status);
        for (i, s) in status.iter().enumerate() {
            if s == &LetterStatus::Absent {
                word.chars[i] = None;
            }
        }
        word
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &c in &self.chars {
            match c {
                Some(ch) => write!(f, "{}", ch.to_uppercase())?,
                None => break,
            }
        }
        Ok(())
    }
}

use colored::Colorize;
impl fmt::Display for Guess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = decode_status(self.status);
        for (i, s) in status.iter().enumerate() {
            let ch = match self.word.chars[i] {
                Some(ch) => ch.to_uppercase().to_string(),
                None => "_".to_string(),
            };
            match s {
                LetterStatus::Absent => write!(f, "{}", ch.to_string().on_black())?,
                LetterStatus::Misplaced => write!(f, "{}", ch.to_string().on_yellow())?,
                LetterStatus::Correct => write!(f, "{}", ch.to_string().on_green())?,
            }
        }
        Ok(())
    }
}

pub fn encode_status(status: &[LetterStatus; NLETTER]) -> u8 {
    status
        .iter()
        .enumerate()
        .map(|(i, x)| 3_u8.pow(i as u32) * *x as u8)
        .sum()
}

pub fn decode_status(encoded: u8) -> [LetterStatus; NLETTER] {
    let mut status = [LetterStatus::Absent; NLETTER];

    for (i, item) in status.iter_mut().enumerate() {
        let pow = 3_u8.pow(i as u32);
        let value = encoded / pow % 3;
        *item = match value {
            0 => LetterStatus::Absent,
            1 => LetterStatus::Misplaced,
            2 => LetterStatus::Correct,
            _ => panic!("Invalid encoding"),
        };
    }
    status
}

pub fn create_word_from_string(word: &str) -> Word {
    let mut res = Word::new();
    for (i, letter) in word.chars().enumerate() {
        res.set_letter(Some(letter), i);
    }
    res
}

#[cfg(test)]
mod tests {

    use super::*;
    use LetterStatus::*;

    #[test]
    fn test_encode_status() {
        assert_eq!(encode_status(&[Absent, Absent, Absent, Absent, Absent]), 0);
        assert_eq!(
            encode_status(&[Misplaced, Absent, Absent, Absent, Absent]),
            1
        );
        assert_eq!(
            encode_status(&[Misplaced, Absent, Misplaced, Absent, Absent]),
            10
        );
        assert_eq!(
            encode_status(&[Correct, Correct, Correct, Correct, Correct]),
            242
        );
        assert_eq!(
            encode_status(&[Correct, Correct, Misplaced, Correct, Correct]),
            233
        );
    }

    #[test]
    fn test_decode_status() {
        assert_eq!(decode_status(0), [Absent, Absent, Absent, Absent, Absent]);
        assert_eq!(
            decode_status(1),
            [Misplaced, Absent, Absent, Absent, Absent]
        );
        assert_eq!(
            decode_status(10),
            [Misplaced, Absent, Misplaced, Absent, Absent]
        );
        assert_eq!(
            decode_status(242),
            [Correct, Correct, Correct, Correct, Correct]
        );
        assert_eq!(
            decode_status(233),
            [Correct, Correct, Misplaced, Correct, Correct]
        );
    }

    #[test]
    fn compare_words() {
        let word = create_word_from_string("water");

        let guess = create_word_from_string("slate");
        let expected = [Absent, Absent, Misplaced, Misplaced, Misplaced];
        assert_eq!(word.compare(&guess), expected);

        let guess = create_word_from_string("eerie");
        let expected = [Misplaced, Absent, Misplaced, Absent, Absent];
        assert_eq!(word.compare(&guess), expected);

        let guess = create_word_from_string("eater");
        let expected = [Absent, Correct, Correct, Correct, Correct];
        assert_eq!(word.compare(&guess), expected);

        let word = create_word_from_string("abide");
        let guess = create_word_from_string("speed");
        let expected = [Absent, Absent, Misplaced, Absent, Misplaced];
        assert_eq!(word.compare(&guess), expected);

        let word = create_word_from_string("erase");
        let guess = create_word_from_string("speed");
        let expected = [Misplaced, Absent, Misplaced, Misplaced, Absent];
        assert_eq!(word.compare(&guess), expected);

        let word = create_word_from_string("steal");
        let guess = create_word_from_string("speed");
        let expected = [Correct, Absent, Correct, Absent, Absent];
        assert_eq!(word.compare(&guess), expected);

        let word = create_word_from_string("crepe");
        let guess = create_word_from_string("speed");
        let expected = [Absent, Misplaced, Correct, Misplaced, Absent];
        assert_eq!(word.compare(&guess), expected);
    }

    #[test]
    fn test_is_valid() {
        let guess = Guess::new("slate", [Absent, Correct, Correct, Correct, Correct]);
        assert!(create_word_from_string("plate").is_valid(&guess));
        assert!(!create_word_from_string("water").is_valid(&guess));

        let guess = Guess::new("esses", [Misplaced, Absent, Absent, Absent, Absent]);
        assert!(!create_word_from_string("reede").is_valid(&guess));

        let guess = Guess::new("slate", [Absent, Misplaced, Correct, Absent, Absent]);
        assert!(!create_word_from_string("least").is_valid(&guess));
    }
}
