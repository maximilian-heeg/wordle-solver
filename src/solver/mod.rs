pub mod letter;
pub mod word;

use letter::*;
use word::Word;

use itertools::Itertools;
use rayon::prelude::*;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{prelude::*, BufReader};

#[derive(Debug, Clone)]
pub struct Solver {
    /// A Vector that contains all words
    pub words: Vec<Word>,
    /// The indices of the remaining words
    remaining_words: Vec<usize>,
    /// The Vector with the solutions
    /// the index of the vector is the index of the word
    /// the value is a  hashmap.
    /// in this, each key is the combination of the letter status
    /// and the value is a vector of possible solutions
    mappings: Vec<Vec<Vec<u16>>>,
    remaining_mappings: Vec<Vec<Vec<u16>>>,
    last_guesses: Vec<Word>,
    cache_mode: CacheMode,
}

#[derive(Debug, Clone, PartialEq, clap::ValueEnum)]
#[clap(rename_all = "kebab_case")]
pub enum CacheMode {
    Words,
    Guesses,
}

fn u16_from_usize(i: usize) -> u16 {
    u16::try_from(i).expect("Too big to fit into u16")
}
const WORDS: &[u8; 89130] = include_bytes!("../../data/words.txt");

impl Solver {
    pub fn new(filepath: Option<&str>, cache_mode: CacheMode) -> Self {
        // Vector to store parsed words
        let mut words: Vec<Word> = Vec::new();

        let reader: Box<dyn std::io::BufRead> = if let Some(path) = filepath {
            let file = match File::open(path) {
                Ok(file) => file,
                Err(why) => panic!("Couldn't open file: {}", why),
            };
            Box::new(BufReader::new(file))
        } else {
            Box::new(BufReader::new(&WORDS[..]))
        };

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

        let mappings = Solver::build_mappings(&words);

        Solver {
            words: words.clone(),
            remaining_words: (0..(words.len())).collect(),
            mappings: mappings.clone(),
            remaining_mappings: mappings,
            last_guesses: vec![],
            cache_mode,
        }
    }

    pub fn reset(&mut self) {
        self.remaining_words = (0..(self.words.len())).collect();
        self.remaining_mappings = self.mappings.clone();
    }

    pub fn build_mappings(words: &Vec<Word>) -> Vec<Vec<Vec<u16>>> {
        let hm: Vec<Vec<Vec<u16>>> = words
            .par_iter()
            .map(|word| Solver::get_possible_solutions_for_word(word, words))
            .collect();
        hm
    }

    fn get_possible_solutions_for_word(word: &Word, all_words: &Vec<Word>) -> Vec<Vec<u16>> {
        let patterns: Vec<[Status; 5]> = all_words
            .par_iter()
            .map(|solution| solution.compare(word))
            .collect();

        let mut hm: HashMap<[Status; 5], Vec<u16>> = HashMap::new();
        patterns.iter().enumerate().for_each(|(i, code)| {
            hm.entry(*code)
                .and_modify(|vec| vec.push(u16_from_usize(i)))
                .or_insert(vec![u16_from_usize(i)]);
        });

        hm.into_values().collect()
    }

    pub fn get_n_remaining_words(&self) -> usize {
        self.remaining_words.len()
    }

    pub fn get_remaining_words(&self) -> Vec<Word> {
        let remaining_words: Vec<Word> = self
            .remaining_words
            .iter()
            .filter_map(|&index| self.words.get(index))
            .copied()
            .collect();
        remaining_words
    }

    fn keep_word(word: &Word, guesses: &[Word]) -> bool {
        guesses.par_iter().all(|guess| word.is_valid(guess))
    }

    pub fn update_remaining_words(&mut self, guesses: &[Word]) {
        if self.cache_mode == CacheMode::Guesses {
            // are the old guesses a subset of the new guesses?
            if !self
                .last_guesses
                .iter()
                .all(|old| guesses.iter().contains(old))
            {
                // reset remaing words and mappging
                self.reset()
            }
            self.last_guesses = guesses.to_vec();
        }

        // if in words cache mode, create a hashset
        let mut a_set: HashSet<usize> = HashSet::new();
        if self.cache_mode == CacheMode::Words {
            a_set = self.remaining_words.iter().copied().collect();
            self.remaining_words = (0..self.words.len()).collect();
        }

        let new_remaining_words: Vec<usize> = self
            .remaining_words
            .par_iter()
            .filter(|&id| Solver::keep_word(&self.words[*id], guesses))
            .map(|x| *x)
            .collect();

        // if in words cache mode, check if the new remaining words are a subset of the old
        if self.cache_mode == CacheMode::Words
            && !new_remaining_words.iter().all(|item| a_set.contains(item))
        {
            self.remaining_mappings = self.mappings.clone();
        }
        self.remaining_words = new_remaining_words;

        self.update_mappings();
    }

    fn update_mappings(&mut self) {
        let remaining_words_set: std::collections::HashSet<_> =
            self.remaining_words.iter().cloned().collect();

        self.remaining_mappings.par_iter_mut().for_each(|word| {
            word.iter_mut().for_each(|v| {
                v.retain(|x| remaining_words_set.contains(&usize::from(*x)));
            });
            word.retain(|x| !x.is_empty());
        });
    }

    pub fn guess(&self, n: usize) -> Vec<Word> {
        if self.get_n_remaining_words() == 1 {
            return self.get_remaining_words();
        }

        let mut hm: HashMap<usize, Vec<usize>> = HashMap::new();
        self.remaining_mappings
            .iter()
            .enumerate()
            .for_each(|(i, word)| {
                let sum = word.iter().filter(|value| !value.is_empty()).count();
                hm.entry(sum).or_default().push(i);
            });

        let mut sorted_keys: Vec<&usize> = hm.keys().collect();
        sorted_keys.sort_by(|a, b| b.cmp(a));

        let mut highest_indices: Vec<usize> = vec![];
        for key in sorted_keys {
            if let Some(idx) = hm.get(key) {
                let mut idx = idx.clone();

                // Sort the by variance of the possibliies in group.
                // eg. a guess that makes two groups of 5 solutions is better
                // than a guess that makes two groups of 9 and 1 solutions
                // Sort so that the value that are possible solutions are first
                idx.sort_by_cached_key(|i| {
                    let mean_idx_per_group: f64 = self.remaining_mappings[*i]
                        .iter()
                        .map(|x| x.len() as f64)
                        .sum::<f64>()
                        / *key as f64;
                    let mean_error = self.remaining_mappings[*i]
                        .iter()
                        .filter(|x| !x.is_empty())
                        .map(|x| (x.len() as f64 - mean_idx_per_group).powf(2.0))
                        .sum::<f64>()
                        * 100.0;
                    let mean_error = mean_error.round() as usize;

                    (mean_error, !self.remaining_words.contains(i))
                });
                highest_indices.extend(idx.iter().take(n - highest_indices.len()));
            }
            if highest_indices.len() >= n {
                break;
            }
        }

        highest_indices.into_iter().map(|i| self.words[i]).collect()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_word_from_string(word: &str) -> Word {
        let mut res = Word::new();
        for (i, letter) in word.chars().enumerate() {
            res.set_letter(Some(letter), i);
        }
        res
    }

    #[test]
    fn test_build_mappings() {
        let solver = Solver::new(Some("data/test.txt"), super::CacheMode::Words);

        let lengths: Vec<usize> = solver.mappings.iter().map(|x| x.len()).collect();

        let should = vec![5, 4, 4, 5, 5];

        assert_eq!(lengths, should)
    }

    #[test]
    fn test_build_mappings_2() {
        let solver = Solver::new(Some("data/words.txt"), super::CacheMode::Words);

        let lengths: Vec<usize> = solver.mappings.iter().map(|x| x.len()).collect();

        let should = vec![142, 132, 127, 161, 130];

        assert_eq!(lengths[1000..1005], should)
    }

    #[test]
    fn test_update_mappings() {
        let mut solver = Solver::new(Some("data/test.txt"), super::CacheMode::Words);

        solver.remaining_words = vec![3, 4];

        solver.remaining_mappings = vec![
            vec![vec![1, 2, 3], vec![3, 4, 5]],
            vec![vec![3], vec![1, 2, 5]],
        ];

        solver.update_mappings();

        let should: Vec<Vec<Vec<u16>>> = vec![vec![vec![3], vec![3, 4]], vec![vec![3]]];

        assert_eq!(solver.remaining_mappings, should)
    }

    #[test]
    fn test_update_remaining_words() {
        let mut solver = Solver::new(Some("data/test.txt"), super::CacheMode::Words);

        let mut guess = create_word_from_string("sport");
        guess.letters[0].status = Status::Misplaced;
        guess.letters[1].status = Status::Absent;
        guess.letters[2].status = Status::Absent;
        guess.letters[3].status = Status::Misplaced;
        guess.letters[4].status = Status::Misplaced;

        solver.update_remaining_words(&[guess]);

        assert_eq!(solver.remaining_words, vec![3])
    }
}
