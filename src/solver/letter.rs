#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Status {
    Unknown,
    Absent,
    Misplaced,
    Correct,
}

#[derive(Copy, Clone, Debug, PartialEq)]
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
}
