use std::io::{self, stdout, Stdout};

use crate::wordlebot::solver::*;
use crate::wordlebot::wordle::*;

use crossterm::{execute, terminal::*};
use ratatui::prelude::*;
use tokio::sync::mpsc;

use actions::Action;
use tokio_util::sync::CancellationToken;

mod actions;
mod events;
mod ui;

const N_SUGGESTIONS: usize = 15;

/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn initialize_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        restore().unwrap();
        original_hook(panic_info);
    }));
}

/// Initialize the terminal
pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

/// Restore the terminal to its original state
pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub struct App {
    exit: bool,
    two_level: bool,
    guesses: [Guess; 6],
    cached_guesses: [Guess; 6],
    selected_word: usize,
    selected_letter: usize,
    solver: Solver,
    remaining_words: Vec<usize>,
    suggestions: Vec<GuessEvaluation>,
    evaludations: Vec<GuessEvaluation>,
    action_tx: mpsc::UnboundedSender<Option<Action>>,
    action_rx: mpsc::UnboundedReceiver<Option<Action>>,
    token: CancellationToken,
    child_token: Option<CancellationToken>,
}

impl App {
    pub fn init(solver: Solver, two_level: bool) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let remaining_words = solver.get_frequent_word_idx();
        let suggestions = vec![];

        // Get Suggestions in the background
        action_tx
            .send(Some(Action::GetSuggestions(vec![])))
            .unwrap();

        App {
            exit: false,
            two_level,
            guesses: [Guess::empty(); 6],
            cached_guesses: [Guess::empty(); 6],
            selected_word: 0,
            selected_letter: 0,
            solver,
            remaining_words,
            suggestions,
            action_rx,
            action_tx,
            token: CancellationToken::new(),
            child_token: None,
            evaludations: vec![],
        }
    }

    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        let task = self.handle_events(self.action_tx.clone());

        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;

            if let Some(action) = self.action_rx.recv().await {
                self.update(action);
            }
        }
        task.abort();
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }
}
