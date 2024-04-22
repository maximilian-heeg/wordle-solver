use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::HashMap;
use wordlebot::{
    self,
    solver::*,
    wordle::{create_word_from_string, decode_status, Guess, LetterStatus::*, Word},
};

mod deep;
mod tui;

/// Wordle solver
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[command(subcommand)]
    command: Option<Commands>,

    // Two level entropy calculation
    #[arg(short, long)]
    two_level: bool,
}

#[derive(Args, Debug)]
struct CliArgs {
    /// Choose a manual starting word
    #[arg(short, long)]
    starting_word: Option<String>,

    /// Maximal number of rounds
    #[arg(short, long, default_value_t = 6)]
    max_rounds: usize,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Default. Launch with graphical interface
    Tui {},

    /// Deep search
    Deep {},

    /// Benchmark against all words in file
    Benchmark {
        #[command(flatten)]
        cli_args: CliArgs,
    },

    /// Get the best strategy to solve words
    Solve {
        /// The words to solve
        words: Vec<String>,

        #[command(flatten)]
        cli_args: CliArgs,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();

    println!(
        "{}",
        "Initializing solver. This might take a while...".blue()
    );
    let solver = wordlebot::solver::Solver::new().context("Error initializing solver")?;

    match args.command {
        Some(Commands::Tui {}) | None => {
            tui::initialize_panic_handler();
            let mut terminal = tui::init()?;
            let app_result = tui::App::init(solver, args.two_level)
                .run(&mut terminal)
                .await;
            tui::restore()?;
            println!("{}", "Shutting down...".blue());
            app_result?;
            Ok(())
        }
        Some(Commands::Deep {}) => {
            deep::deep(&solver);
            Ok(())
        }
        Some(Commands::Benchmark { cli_args }) => {
            let starting_word = pick_starting_word(cli_args.starting_word, &solver);
            benchmark(&solver, cli_args.max_rounds, starting_word);
            Ok(())
        }
        Some(Commands::Solve { cli_args, words }) => {
            use std::time::Instant;
            let starting_word = pick_starting_word(cli_args.starting_word, &solver);
            for word in words {
                let now = Instant::now();
                let word = create_word_from_string(&word);
                try_to_solve(&word, &solver, cli_args.max_rounds, true, starting_word);
                let elapsed = now.elapsed();
                println!(" --- Elapsed: {:.2?}", elapsed);
            }
            Ok(())
        }
    }
}

fn pick_starting_word(word: Option<String>, solver: &Solver) -> Word {
    match word {
        Some(word) => create_word_from_string(&word),
        None => solver.guess(1, &solver.get_frequent_word_idx(), 0.0)[0],
    }
}

fn benchmark(solver: &Solver, max_rounds: usize, start: Word) {
    let words = solver.get_words_from_idx(&solver.get_frequent_word_idx());

    println!("Starting benchmark.");
    let style =
        ProgressStyle::with_template("{wide_bar} {pos:>7}/{len:7} [{eta_precise} remaining]")
            .unwrap()
            .progress_chars("##-");
    let mut steps: Vec<usize> = words
        .par_iter()
        .progress_with_style(style)
        .map(|word| try_to_solve(word, solver, max_rounds, false, start))
        .collect();

    let failed = steps.iter().filter(|&x| *x == (0_usize)).count();
    let failes_idx: Vec<usize> = steps
        .iter()
        .enumerate()
        .filter(|(_, &x)| x == (0_usize))
        .map(|(id, _)| id)
        .collect();
    let failed_words = solver
        .get_words_from_idx(&failes_idx)
        .into_iter()
        .map(|i| format!("{}", i))
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

fn print_guess_evaludation(guess: &Guess, remaining_words: &[usize], solver: &Solver) {
    let res = solver.evalute_guess(
        &guess.word,
        remaining_words,
        Some(decode_status(guess.status)),
        false,
    );
    println!(
        " {} - n before: {:4?} | n after: {:4?} | bits {:.2} | n groups {:3} | max group {:4}",
        guess,
        res.n_remaining_before,
        res.n_remaining_after.unwrap(),
        res.expected_bits,
        res.groups,
        res.max_group_size
    )
}

fn try_to_solve(
    word: &Word,
    solver: &Solver,
    max_rounds: usize,
    print: bool,
    start: Word,
) -> usize {
    let mut guesses: Vec<Guess> = vec![];
    let status = word.compare(&start);
    guesses.push(Guess::from_word(start, status));
    if print {
        println!(
            "{}",
            format!(
                "Trying to solve {}",
                format!("{}", word).bold().bright_magenta()
            )
            .underline()
        );
        print_guess_evaludation(
            guesses.last().unwrap(),
            &solver.get_frequent_word_idx(),
            solver,
        )
    };
    if status.iter().all(|s| *s == Correct) {
        return 1;
    }

    for step in 2..=max_rounds {
        let remaining_idx = solver.get_remaining_words_idx(&guesses);

        let penalty = 0.1;
        let next_guess = solver.guess(1, &remaining_idx, penalty)[0];

        let status = word.compare(&next_guess);
        guesses.push(Guess::from_word(next_guess, status));

        if print {
            print_guess_evaludation(guesses.last().unwrap(), &remaining_idx, solver)
        };
        if status.iter().all(|s| *s == Correct) {
            return step;
        }
    }
    0
}
