use anyhow::{Context, Result};
use std::io::{prelude::*, BufReader};

use crate::wordle::{create_word_from_string, Word};

pub const N_LINES: usize = 14855;

const DATA: &[u8] = include_bytes!("../../data/words.csv");

pub fn import() -> Result<([Word; N_LINES], [f32; N_LINES])> {
    let mut words = [Word::new(); N_LINES];
    let mut priors: [f32; N_LINES] = [0.0; N_LINES];

    let reader = BufReader::new(DATA);
    for (i, line) in reader.lines().skip(1).enumerate() {
        let line = line.context("Error reading line")?;

        let cells: Vec<&str> = line.split('\t').collect();
        // Add the word to the vector
        words[i] = create_word_from_string(cells[0]);
        priors[i] = cells[1].parse::<f32>().context("Parsing prior")?;
    }
    Ok((words, priors))
}
