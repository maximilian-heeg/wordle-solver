use crate::app::letter::*;
use crate::app::word::*;

use itertools::Itertools;
use rayon::prelude::*;

use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};

pub struct Solver {
    pub words: Vec<Word>,
    remaining_words: Vec<Word>,
    scores_frequency: Vec<usize>,
    scores_groups: Vec<usize>,
}

static WORDLE_SOLUTIONS: &str = r"data/words.txt";

impl Solver {
    pub fn new() -> Self {
        // Vector to store parsed words
        let mut words: Vec<Word> = Vec::new();

        let file = match File::open(WORDLE_SOLUTIONS) {
            Ok(file) => file,
            Err(why) => panic!("Couldn't open file: {}", why),
        };

        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.expect("Error reading line");

            // Create a word and fill it with letters
            let mut word = Word {
                letters: [Letter {
                    letter: None,
                    status: Status::Unknown,
                }; 5],
            };
            for (i, c) in line.chars().enumerate().take(5) {
                word.letters[i].letter = Some(c);
            }

            // Add the word to the vector
            words.push(word);
        }

        Solver {
            words: words.clone(),
            remaining_words: words,
            scores_groups: vec![],
            scores_frequency: vec![],
        }
    }

    pub fn get_n_remaining_words(&self) -> usize {
        self.get_remaining_words().len()
    }

    pub fn get_remaining_words(&self) -> &Vec<Word> {
        &self.remaining_words
    }

    fn filter_words(words: &mut Vec<Word>, guess: Word) {
        words.retain(|word| word.keep_word(&guess));
    }

    pub fn update_remaining_words(&mut self, guesses: &[Word]) {
        let mut words = self.words.clone();

        for guess in guesses.iter() {
            Self::filter_words(&mut words, *guess);
        }
        self.remaining_words = words;

        self.calculate_word_score_frequency();
        self.calculate_word_score_groups();
    }

    fn create_frequency_hashmap(&self, solutions: &[Word]) -> [HashMap<char, usize>; 5] {
        let mut letter_map: [HashMap<char, usize>; 5] = [
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
        ];

        solutions.iter().for_each(|word| {
            word.letters.iter().enumerate().for_each(|(i, letter)| {
                if let Some(char) = letter.letter {
                    letter_map[i]
                        .entry(char)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
            });
        });
        letter_map
    }

    fn word_score_frequency(word: &Word, letter_map: &[HashMap<char, usize>; 5]) -> usize {
        word.letters
            .iter()
            .enumerate()
            .map(|(i, letter)| match letter.letter {
                Some(c) => match letter_map[i].get(&c) {
                    Some(v) => *v,
                    None => 0,
                },
                None => 0,
            })
            .sum()
    }

    fn calculate_word_score_frequency(&mut self) {
        let solutions = &self.words;
        let letter_map = self.create_frequency_hashmap(self.get_remaining_words());
        let scores: Vec<usize> = solutions
            .iter()
            .map(|word| Self::word_score_frequency(word, &letter_map))
            .collect();
        self.scores_frequency = scores;
    }

    pub fn next_guess_frequency(&self, n: usize) -> Vec<Word> {
        let solutions = &self.words;
        let scores = &self.scores_frequency;

        let mut indexed_vec: Vec<(usize, &usize)> = scores.iter().enumerate().collect();
        // Sort the vector of indices based on the values
        indexed_vec.sort_by_key(|&(_i, val)| std::cmp::Reverse(val));
        let highest_indices: Vec<usize> = indexed_vec.iter().take(n).map(|&(i, _)| i).collect();

        highest_indices.into_iter().map(|i| solutions[i]).collect()
    }

    fn calculate_word_score_group(word: &Word, solutions: &[Word]) -> usize {
        let patterns: Vec<Vec<Status>> = solutions
            .par_iter()
            .map(|solution| solution.compare(word))
            .collect();

        patterns.into_iter().unique().count()
    }

    fn calculate_word_score_groups(&mut self) {
        let solutions = self.get_remaining_words();
        // let words: Vec<&Word> = self.words.iter().take(100).collect();
        let words = &self.words;
        let scores: Vec<usize> = words
            .par_iter()
            .map(|word| Self::calculate_word_score_group(word, solutions))
            .collect();
        self.scores_groups = scores
    }

    pub fn next_guess_groups(&self, n: usize) -> Vec<Word> {
        // let words: Vec<&Word> = self.words.iter().take(100).collect();
        let words = &self.words;
        let scores = &self.scores_groups;
        // let scores_freq = cal

        let mut indexed_vec: Vec<(usize, &usize)> = scores.iter().enumerate().collect();
        // Sort the vector of indices based on the values
        indexed_vec.sort_by_key(|&(_i, val)| std::cmp::Reverse(val));
        let highest_indices: Vec<usize> = indexed_vec.iter().take(n).map(|&(i, _)| i).collect();

        highest_indices.into_iter().map(|i| words[i]).collect()
    }

    fn scale_vector_to_unit_interval(vector: &[usize]) -> Vec<f64> {
        if vector.is_empty() {
            return vec![]; // Return an empty vector if the input is empty
        }

        // Find the minimum and maximum values in the vector
        let min_value = *vector.iter().min().unwrap() as f64;
        let max_value = *vector.iter().max().unwrap() as f64;

        if max_value == min_value {
            return vector.iter().map(|&x| x as f64).collect();
        }

        // Scale each value to the range [0, 1]
        let scaled_vector: Vec<f64> = vector
            .iter()
            .map(|&x| ((x as f64 - min_value) / (max_value - min_value)))
            .collect();

        scaled_vector
    }

    pub fn next_guess(&self, n: usize, guess_number: usize) -> Vec<Word> {
        let words = &self.words;
        let scores_group = &self.scores_groups;
        let scores_freq = &self.scores_frequency;

        let scores_group = Self::scale_vector_to_unit_interval(scores_group);
        let scores_freq = Self::scale_vector_to_unit_interval(scores_freq);

        let combined: Vec<f64> = scores_group
            .iter()
            .zip(scores_freq.iter())
            .map(|(group, freq)| group + guess_number as f64 * freq)
            .collect();

        let mut indexed_vec: Vec<(usize, &f64)> = combined.iter().enumerate().collect();
        // Sort the vector of indices based on the values
        indexed_vec.sort_by(|&(_, a), &(_, b)| b.partial_cmp(a).unwrap());
        let highest_indices: Vec<usize> = indexed_vec.iter().take(n).map(|&(i, _)| i).collect();

        highest_indices.into_iter().map(|i| words[i]).collect()
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
    fn group_count() -> io::Result<()> {
        let possible_solutions = vec![
            create_word_from_string("slate"),
            create_word_from_string("plate"),
            create_word_from_string("water"),
        ];

        let guess = create_word_from_string("slate");
        assert_eq!(
            Solver::calculate_word_score_group(&guess, &possible_solutions),
            3
        );

        let guess = create_word_from_string("penny");
        assert_eq!(
            Solver::calculate_word_score_group(&guess, &possible_solutions),
            2
        );
        Ok(())
    }
}
