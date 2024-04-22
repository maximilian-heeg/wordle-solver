use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use wordlebot::{
    solver::{GuessEvaluation, Solver},
    wordle::{decode_status, Guess},
};

pub fn deep(solver: &Solver) {
    let remaining_words = solver.get_remaining_words_idx(&[]);
    recurse(solver, &[], &remaining_words);
}

fn recurse(solver: &Solver, guesses: &[Guess], remaining_words: &[usize]) {
    let words = solver.guess(20, remaining_words, 0.0);

    let scores: Vec<f32> = words
        .par_iter()
        .map(|word| {
            let eval = solver.evalute_guess(word, remaining_words, None, false);
            let avg_bits = avg_bits(solver, &eval, guesses);
            avg_bits + eval.expected_bits
        })
        .collect();

    let mut res: Vec<_> = words.iter().zip(scores).collect();

    res.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());

    for (word, score) in res {
        println!("{} {}", word, score);
    }
}

/// This function calculates the avg bits of information
/// for all next guesses of a guess
fn avg_bits(solver: &Solver, eval: &GuessEvaluation, guesses: &[Guess]) -> f32 {
    let mut guess = Guess::from_word(eval.word, decode_status(0));
    let avg_bits: f32 = eval
        .group_probabilities
        .iter()
        .map(|(status, prop)| {
            guess.set_status(&decode_status(*status));
            let mut new_guesses = guesses.to_vec();
            new_guesses.push(guess);
            let remaining_words = solver.get_remaining_words_idx(&new_guesses);
            let next = solver.guess(1, &remaining_words, 0.1)[0];
            let next_eval = solver.evalute_guess(&next, &remaining_words, None, false);
            *prop * next_eval.expected_bits
        })
        .sum();
    avg_bits
}
