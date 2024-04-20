use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::solver::data::import;
use crate::wordle::*;
use anyhow::{Context, Result};
use ndarray::{prelude::*, Zip};

pub mod data;

pub struct Solver {
    // An array of words
    words: Vec<Word>,

    // An array of priors
    // the indices are the same as for words
    priors: Vec<f32>,

    // The mappings between all words
    // row and column inidces are the indices for words
    // the values in the u8 encoded pattern
    mappings: Array<u8, Ix2>,
}

fn create_mappings(words: &[Word]) -> Array<u8, Ix2> {
    let mut mappings: Array<u8, Ix2> = Array::zeros((words.len(), words.len()));
    Zip::indexed(&mut mappings)
        .par_for_each(|(i, j), val| *val = encode_status(&words[j].compare(&words[i])));

    mappings
}

fn entropy(x: &ArrayView<f32, Ix1>) -> f32 {
    let sum: f32 = x.iter().sum();
    x.iter()
        .map(|&v| {
            if v == 0.0 {
                return 0.0;
            }
            let p = v / sum;
            -p * f32::log2(p)
        })
        .sum()
}

fn get_group_size(id: usize, distributions: &Array<f32, Ix2>) -> usize {
    let distribution = distributions.row(id);
    distribution.iter().filter(|&x| *x > 0.0).count()
}

fn rank_guess(entropy: f32, prior: f32, penalty: f32, possible: bool) -> f32 {
    if !possible {
        return entropy;
    }
    entropy + prior / 20. * penalty
}

impl Solver {
    pub fn new() -> Result<Solver> {
        let (words, priors) = import().context("Error importing data")?;
        let mappings = create_mappings(&words);
        Ok(Solver {
            words: words.into(),
            priors: priors.into(),
            mappings,
        })
    }

    /// Allowed words are the allowed guesses, eg, 14000 words
    fn get_mapping_distribution(
        &self,
        allowed_words: &[usize],
        remaining_words: &[usize],
    ) -> Array<f32, Ix2> {
        let pattern_matrix = self
            .mappings
            .select(Axis(1), remaining_words)
            .select(Axis(0), allowed_words);
        let n = allowed_words.len();
        let mut distributions: Array<f32, Ix2> = Array::zeros((n, 3_usize.pow(5)));
        let n_range: Vec<usize> = (0..n).collect::<Vec<usize>>();
        pattern_matrix
            .axis_iter(Axis(1))
            .enumerate()
            .for_each(|(id, column)| {
                column
                    .iter()
                    .zip(&n_range)
                    .for_each(|(&j, i)| distributions[[*i, j as usize]] += self.priors[id]);
            });
        distributions
    }

    pub fn get_remaining_words_idx(&self, guesses: &[Guess]) -> Vec<usize> {
        let frequent_words = self.get_frequent_word_idx();
        if guesses.is_empty() {
            return frequent_words;
        }
        let res: Vec<usize> = guesses
            .iter()
            .map(|g| {
                let id = self
                    .words
                    .iter()
                    .position(|&r| r == g.word)
                    .expect("Not a valid guess");

                self.mappings
                    .row(id)
                    .iter()
                    .enumerate()
                    .filter(|(_, &x)| x == g.status)
                    .map(|(i, _)| i)
                    .collect::<Vec<usize>>()
            })
            .map(HashSet::from_iter)
            .reduce(|a: HashSet<usize>, b| a.intersection(&b).cloned().collect())
            .unwrap()
            .intersection(&HashSet::from_iter(frequent_words))
            .copied()
            .collect();
        res
    }

    pub fn get_words_from_idx(&self, idx: &[usize]) -> Vec<Word> {
        idx.iter().map(|&i| self.words[i]).collect()
    }

    pub fn evalute_guess(
        &self,
        word: &Word,
        remaining_words: &[usize],
        status: Option<[LetterStatus; 5]>,
    ) -> GuessEvaluation {
        let word_id = self
            .words
            .iter()
            .position(|w| word == w)
            .expect("Not a valid guess");

        let distributions = self.get_mapping_distribution(&[word_id], remaining_words);

        let entropies: Vec<f32> = distributions
            .map_axis(Axis(1), |x| entropy(&x))
            .iter()
            .copied()
            .collect();

        let n_after =
            status.map(|status| self.get_n_solutions_after_guess(word_id, remaining_words, status));

        let real_bits = n_after.map(|x| f32::log2(remaining_words.len() as f32 / x as f32));

        GuessEvaluation {
            word: *word,
            expected_bits: entropies[0],
            real_bits,
            groups: get_group_size(0, &distributions),
            max_group_size: self.get_max_group_size(word_id, remaining_words),
            n_remaining_before: remaining_words.len(),
            n_remaining_after: n_after,
            is_possible: remaining_words.contains(&word_id),
            prior: self.priors[word_id],
        }
    }

    fn get_n_solutions_after_guess(
        &self,
        word_id: usize,
        remaining_words: &[usize],
        status: [LetterStatus; 5],
    ) -> usize {
        let possible_word_ids = self
            .mappings
            .row(word_id)
            .iter()
            .enumerate()
            .filter(|(_, &x)| x == encode_status(&status))
            .map(|(i, _)| i)
            .collect::<Vec<usize>>();
        // Convert vectors into sets
        let set1: HashSet<_> = remaining_words.iter().collect();
        let set2: HashSet<_> = possible_word_ids.iter().collect();

        // Find the intersection of the two sets
        let intersection: HashSet<_> = set1.intersection(&set2).collect();
        intersection.len()
    }

    fn get_max_group_size(&self, word_id: usize, remaining_words: &[usize]) -> usize {
        let pattern_matrix = self.mappings.row(word_id).select(Axis(0), remaining_words);
        let mut frequency_map = HashMap::new();

        pattern_matrix.iter().for_each(|num| {
            *frequency_map.entry(num).or_insert(0) += 1;
        });
        let max_frequency = frequency_map.values().cloned().max().unwrap_or(0);
        max_frequency
    }

    pub fn guess(&self, n: usize, remaining_words: &[usize], pentalty: f32) -> Vec<Word> {
        if remaining_words.len() == 1 {
            return remaining_words.iter().map(|&i| self.words[i]).collect();
        }

        let is_in_remaining: Vec<bool> = (0..self.words.len())
            .map(|x| remaining_words.contains(&x))
            .collect();

        let distributions = self.get_mapping_distribution(
            &(0..self.words.len()).collect::<Vec<usize>>(),
            remaining_words,
        );

        let entropies: Vec<f32> = distributions
            .map_axis(Axis(1), |x| entropy(&x))
            .iter()
            .copied()
            .collect();

        let mut indices: Vec<usize> = (0..self.words.len()).collect();
        // indices.sort_by_cached_key(|i| (Reverse(entropies[*i])));
        indices.sort_by(|&a, &b| {
            rank_guess(entropies[b], self.priors[b], pentalty, is_in_remaining[b])
                .partial_cmp(&rank_guess(
                    entropies[a],
                    self.priors[a],
                    pentalty,
                    is_in_remaining[a],
                ))
                .unwrap()
        });

        let highest_indices: Vec<usize> = indices.iter().take(n).cloned().collect();

        highest_indices.iter().map(|&i| self.words[i]).collect()
    }

    pub fn get_frequent_word_idx(&self) -> Vec<usize> {
        self.priors
            .iter()
            .enumerate()
            .filter(|(_, &x)| x > 0.0)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn is_valid_guess(&self, word: &Word) -> bool {
        self.words.contains(word)
    }
}

#[derive(Clone, Copy)]
pub struct GuessEvaluation {
    pub word: Word,
    pub expected_bits: f32,
    pub real_bits: Option<f32>,
    pub groups: usize,
    pub max_group_size: usize,
    pub n_remaining_before: usize,
    pub n_remaining_after: Option<usize>,
    pub is_possible: bool,
    pub prior: f32,
}

impl fmt::Display for GuessEvaluation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (Bits : {:.2}, groups: {})",
            self.word, self.expected_bits, self.groups
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::LetterStatus::*;
    use super::*;
    use approx::*;

    #[test]
    fn test_mappings() {
        let solver = Solver::new().unwrap();

        // The diagonal of the matrix need to be 242 (perfect fit) for
        // all values, since the index and hence the words for x and y is the
        // same
        assert!(solver.mappings.diag().iter().all(|x| *x == 242u8));
    }

    #[test]
    fn test_remaining_words_idx() {
        let solver = Solver::new().unwrap();
        let mut guesses = vec![Guess::new(
            "tares",
            [Misplaced, Correct, Absent, Correct, Absent],
        )];

        let remaining = solver.get_remaining_words_idx(&guesses);
        assert_eq!(remaining.len(), 12);

        guesses.push(Guess::new(
            "dempt",
            [Absent, Misplaced, Absent, Absent, Correct],
        ));
        let remaining = solver.get_remaining_words_idx(&guesses);
        assert_eq!(remaining.len(), 2);
    }

    fn test_solver() -> Solver {
        let words = vec![
            create_word_from_string("slate"),
            create_word_from_string("water"),
            create_word_from_string("goose"),
        ];
        let mappings = create_mappings(&words);
        Solver {
            words,
            priors: vec![1., 1., 1.],
            mappings,
        }
    }

    #[test]
    fn test_mappings_2() {
        let solver = test_solver();
        let expected = array![[242, 117, 163], [39, 242, 27], [189, 81, 242]];
        assert_eq!(solver.mappings, expected)
    }

    #[test]
    fn test_get_mapping_distribution() {
        let solver = test_solver();

        let dist = solver.get_mapping_distribution(&vec![0], &vec![0, 1, 2]);
        let expected = array![
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 1., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 1., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            1.
        ];
        let expected = expected.into_shape([1, 243]).unwrap();
        assert_eq!(dist.shape(), [1, 243]);
        assert_eq!(dist, expected);

        let dist = solver.get_mapping_distribution(&vec![0, 1], &vec![0, 1, 2]);
        assert_eq!(dist.shape(), [2, 243]);
        assert_eq!(dist.index_axis(Axis(0), 0), expected.index_axis(Axis(0), 0));

        let expected = array![
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 1., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 1., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            1.
        ];
        assert_eq!(dist.index_axis(Axis(0), 1), expected);
    }

    #[test]
    fn test_get_mapping_distribution_prior() {
        let mut solver = test_solver();
        solver.priors = vec![1., 2., 3.];

        let dist = solver.get_mapping_distribution(&vec![0], &vec![0, 1, 2]);
        let expected = array![
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 2., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 3., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            1.
        ];
        let expected = expected.into_shape([1, 243]).unwrap();
        assert_eq!(dist.shape(), [1, 243]);
        assert_eq!(dist, expected);
    }

    #[test]
    fn test_entropy() {
        let x = array![1., 2., 3.];
        assert_relative_eq!(entropy(&x.view()), 1.4591479);

        let solver = test_solver();
        let dist = solver.get_mapping_distribution(&vec![0, 1], &vec![0, 1, 2]);
        let entropies: Vec<f32> = dist
            .map_axis(Axis(1), |x| entropy(&x))
            .iter()
            .map(|x| *x)
            .collect();

        assert_eq!(entropies, vec![1.5849626, 1.5849626])
    }

    #[test]
    fn test_step_penalty() {
        let solver = Solver::new().unwrap();

        let guess = solver.guess(1, &solver.get_frequent_word_idx(), 0.0)[0];
        assert_eq!(guess, create_word_from_string("tarse"));

        let guess = solver.guess(1, &solver.get_frequent_word_idx(), 10.0)[0];
        assert_eq!(guess, create_word_from_string("raise"));
    }

    #[test]
    fn test_mapping_subset() {
        let solver = Solver::new().unwrap();
        let dist =
            solver.get_mapping_distribution(&vec![10], &solver.get_remaining_words_idx(&vec![]));
        let dist2 = solver.get_mapping_distribution(
            &(0..solver.words.len()).collect::<Vec<usize>>(),
            &solver.get_remaining_words_idx(&vec![]),
        );
        assert_eq!(dist.row(0), dist2.row(10));
    }

    #[test]
    fn test_guess_evaluation() {
        let solver = Solver::new().unwrap();
        let guess = create_word_from_string("slate");

        let res = solver.evalute_guess(
            &guess,
            &solver.get_frequent_word_idx(),
            Some([Misplaced, Absent, Misplaced, Absent, Correct]),
        );

        assert_eq!(res.groups, 154);
        assert_eq!(res.max_group_size, 328);
        assert_eq!(res.n_remaining_before, 3189);
        assert_eq!(res.n_remaining_after, Some(13));
        assert_relative_eq!(res.expected_bits, 5.789861);
        assert_eq!(res.real_bits, Some(7.938449));
    }
}
