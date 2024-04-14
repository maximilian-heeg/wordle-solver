#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use clap::{Parser, Subcommand};
use colored::Colorize;
use indicatif::ProgressIterator;
use indicatif::ProgressStyle;
use solver::letter::Status;
use std::{collections::HashMap, io};

use solver::{word::Word, Solver};
mod app;
mod solver;
mod tui;

/// Wordle solver
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// No not restrict the possible answers to the
    /// most common words
    #[arg(short, long)]
    no_restriction: bool,

    /// File path to the possible solutions
    #[arg(short, long)]
    word_file: Option<String>,

    /// Maximal number of rounds
    #[arg(short, long, default_value_t = 6)]
    max_rounds: usize,

    /// Cache mode
    #[arg(long, default_value = "words")]
    cache_mode: solver::CacheMode,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Default. Launch with graphical interface
    Tui {},

    /// Benchmark against all words in file
    Benchmark {},

    /// Get the best strategy to solve words
    Solve {
        /// The words to solve
        words: Vec<String>,
    },
}

fn create_word_from_string(word: &str) -> Word {
    let mut res = Word::new();
    for (i, letter) in word.chars().enumerate() {
        res.set_letter(Some(letter), i);
    }
    res
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    println!(
        "{}",
        "Initializing solver. This might take a while...".blue()
    );
    let mut solver = Solver::new(
        args.word_file.as_deref(),
        !args.no_restriction,
        args.cache_mode,
    );

    match args.command {
        Some(Commands::Benchmark {}) => {
            benchmark(&mut solver, args.max_rounds);
            Ok(())
        }
        Some(Commands::Solve { words }) => {
            for word in words {
                let word = create_word_from_string(&word);
                try_to_solve(&word, &mut solver, args.max_rounds, true);
                solver.reset();
            }

            Ok(())
        }
        Some(Commands::Tui {}) | None => {
            let mut terminal = tui::init()?;
            let app_result = app::App::init(solver).run(&mut terminal);
            tui::restore()?;
            app_result
        }
    }
}

fn benchmark(solver: &mut Solver, max_rounds: usize) {
    let words: Vec<Word> = solver
        .answers
        .clone()
        .iter()
        .map(|id| solver.words[*id])
        .collect();

    println!("Starting benchmark.");
    let style =
        ProgressStyle::with_template("{wide_bar} {pos:>7}/{len:7} [{eta_precise} remaining]")
            .unwrap()
            .progress_chars("##-");
    let mut steps: Vec<usize> = words
        .iter()
        .progress_with_style(style)
        .map(|word| try_to_solve(word, solver, max_rounds, false))
        .collect();

    let failed = steps.iter().filter(|&x| *x == (0_usize)).count();
    let failes_idx: Vec<usize> = steps
        .iter()
        .enumerate()
        .filter(|(_, &x)| x == (0_usize))
        .map(|(id, _)| id)
        .collect();
    let failed_words = failes_idx
        .into_iter()
        .map(|i| format!("{}", solver.words[i]))
        .collect::<Vec<String>>()
        .join(", ");
    println!(
        "{} words could not be solved in {} guesses: {}",
        failed, max_rounds, failed_words
    );

    // Step 1: Remove all occurrences of 0 from the vector
    steps.retain(|&x| x != 0);

    // Step 2: Calculate the mean of the remaining values
    let sum: usize = steps.iter().sum();
    let mean: f64 = sum as f64 / steps.len() as f64;

    // Step 3: Count the number of unique values
    let mut counts: HashMap<usize, usize> = HashMap::new();
    // Iterate through the vector and update counts
    for &num in &steps {
        *counts.entry(num).or_insert(0) += 1;
    }

    println!(
        "The others have been solved in an average of {:.2} steps",
        mean
    );
    // Print the counts for each unique value
    println!("Here are the numbers for how many wordles have been solved in n steps.");
    // Get sorted keys
    let mut sorted_keys: Vec<usize> = counts.keys().copied().collect();
    sorted_keys.sort();

    // Print the counts for each unique value in sorted order
    for num in sorted_keys {
        if let Some(count) = counts.get(&num) {
            println!("Steps {}: Count {}", num, count);
        }
    }
}

fn try_to_solve(word: &Word, solver: &mut Solver, max_rounds: usize, print: bool) -> usize {
    let mut guesses: Vec<Word> = vec![];
    if print {
        println!("----- {} -------", format!("{word}").bold().blue());
        println!("Trying to solve in {} rounds", max_rounds)
    };

    // Reset remaining words
    solver.reset();

    for step in 1..(max_rounds + 1) {
        if print {
            println!("Step {}", step.to_string().bold())
        };
        solver.update_remaining_words(&guesses);
        if print {
            println!(
                "   {} remaining words",
                solver.get_n_remaining_words().to_string().bold().blue()
            )
        };
        if solver.get_n_remaining_words() == 1 {
            let next_guess = solver.guess(1)[0];
            if print {
                println!(
                    "   {} is the only remaining word",
                    format!("{}", next_guess.0).blue()
                );
                println!(
                    "{} after {} steps",
                    "Solved".to_string().bold().green(),
                    step.to_string().bold().blue()
                );
                println!();
            };
            return step;
        }
        let mut next_guess = solver.guess(1)[0];
        if print {
            println!("   next guess {}", format!("{}", next_guess.0).blue())
        };
        let status = word.compare(&next_guess.0);
        if status.iter().all(|s| *s == Status::Correct) {
            // We guessed correctly, even if there have been mulipe solutions.
            if print {
                println!(
                    "{} after {} steps",
                    "Solved".to_string().bold().green(),
                    step.to_string().bold().blue()
                );
                println!();
            };
            return step;
        }

        for (i, s) in status.iter().enumerate() {
            next_guess.0.letters[i].status = *s;
        }
        guesses.push(next_guess.0)
    }
    if print {
        println!(
            "{} to solve after {} rounds",
            "Failed".to_string().bold().red(),
            max_rounds.to_string().bold().blue()
        );
        println!(
            "Remaining words: {}",
            solver
                .get_remaining_words()
                .iter()
                .map(|w| format!("{w}"))
                .collect::<Vec<String>>()
                .join(", ")
                .blue()
        );
        println!();
    };
    0
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    #[test]
    fn test_solver_word_cache() -> io::Result<()> {
        let word = create_word_from_string("plaid");
        let mut solver = Solver::new(Some("data/words.txt"), false, solver::CacheMode::Words);

        let mut guesses: Vec<Word> = vec![];
        let mut next_guess = solver.guess(1)[0];
        assert_eq!(format!("{}", next_guess.0), "tares");
        let status = word.compare(&next_guess.0);
        for (i, s) in status.iter().enumerate() {
            next_guess.0.letters[i].status = *s;
        }
        guesses.push(next_guess.0);
        solver.update_remaining_words(&guesses);
        let mut next_guess = solver.guess(1)[0];
        assert_eq!(format!("{}", next_guess.0), "colin");

        let status = word.compare(&next_guess.0);
        for (i, s) in status.iter().enumerate() {
            next_guess.0.letters[i].status = *s;
        }
        guesses.push(next_guess.0);
        solver.update_remaining_words(&guesses);
        let mut next_guess = solver.guess(1)[0];
        assert_eq!(format!("{}", next_guess.0), "plaga");

        let status = word.compare(&next_guess.0);
        for (i, s) in status.iter().enumerate() {
            next_guess.0.letters[i].status = *s;
        }
        guesses.push(next_guess.0);
        solver.update_remaining_words(&guesses);
        let next_guess = solver.guess(1)[0];
        assert_eq!(format!("{}", next_guess.0), "plaid");

        Ok(())
    }

    #[test]
    fn test_solver_guesses_cache() -> io::Result<()> {
        let word = create_word_from_string("sport");
        let mut solver = Solver::new(Some("data/words.txt"), false, solver::CacheMode::Guesses);

        let mut guesses: Vec<Word> = vec![];
        let mut next_guess = solver.guess(1)[0].0;
        assert_eq!(format!("{}", next_guess), "tares");
        let status = word.compare(&next_guess);
        for (i, s) in status.iter().enumerate() {
            next_guess.letters[i].status = *s;
        }
        guesses.push(next_guess);
        solver.update_remaining_words(&guesses);
        let mut next_guess = solver.guess(1)[0].0;
        assert_eq!(format!("{}", next_guess), "spout");

        let status = word.compare(&next_guess);
        for (i, s) in status.iter().enumerate() {
            next_guess.letters[i].status = *s;
        }
        guesses.push(next_guess);
        solver.update_remaining_words(&guesses);
        let next_guess = solver.guess(1)[0].0;
        assert_eq!(format!("{}", next_guess), "sport");
        Ok(())
    }

    #[test]
    fn test_solver_guesses_embedded() -> io::Result<()> {
        let word = create_word_from_string("sport");
        let mut solver = Solver::new(None, false, solver::CacheMode::Guesses);

        let mut guesses: Vec<Word> = vec![];
        let mut next_guess = solver.guess(1)[0].0;
        assert_eq!(format!("{}", next_guess), "tares");
        let status = word.compare(&next_guess);
        for (i, s) in status.iter().enumerate() {
            next_guess.letters[i].status = *s;
        }
        guesses.push(next_guess);
        solver.update_remaining_words(&guesses);
        let mut next_guess = solver.guess(1)[0].0;
        assert_eq!(format!("{}", next_guess), "spout");

        let status = word.compare(&next_guess);
        for (i, s) in status.iter().enumerate() {
            next_guess.letters[i].status = *s;
        }
        guesses.push(next_guess);
        solver.update_remaining_words(&guesses);
        let next_guess = solver.guess(1)[0].0;
        assert_eq!(format!("{}", next_guess), "sport");
        Ok(())
    }
}
