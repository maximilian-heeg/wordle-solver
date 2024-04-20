use std::io::{self, stdout, Stdout};

use crate::wordlebot::solver::*;
use crate::wordlebot::wordle::*;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyEventKind;
use crossterm::{execute, terminal::*};
use ratatui::prelude::*;

mod keyevents;
mod ui;
mod update;

const N_SUGGESTIONS: usize = 20;

/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

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
    guesses: [Guess; 6],
    cached_guesses: [Guess; 6],
    selected_word: usize,
    selected_letter: usize,
    solver: Solver,
    remaining_words: Vec<usize>,
    suggestions: Vec<GuessEvaluation>,
    evaludations: Vec<GuessEvaluation>,
}

impl App {
    pub fn init(solver: Solver) -> Self {
        let remaining_words = solver.get_frequent_word_idx();
        let suggestions = solver
            .guess(N_SUGGESTIONS, &remaining_words, 0.0)
            .iter()
            .map(|w| solver.evalute_guess(w, &remaining_words, None))
            .collect();

        App {
            exit: false,
            guesses: [Guess::empty(); 6],
            cached_guesses: [Guess::empty(); 6],
            selected_word: 0,
            selected_letter: 0,
            solver,
            remaining_words,
            suggestions,
            evaludations: vec![],
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
}
