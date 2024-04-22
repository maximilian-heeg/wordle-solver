use super::*;

pub enum Action {
    Exit,
    MoveLeft,
    MoveRight,
    MoveDown,
    MoveUp,
    Enter,
    EnterChar(char),
    DeleteChar,
    ToggleStatus,
    UpdateGuesses,
    GetSuggestions(Vec<Guess>),
    UpdateSuggestions(Vec<GuessEvaluation>),
}

impl App {
    pub fn update(&mut self, msg: Option<Action>) {
        if let Some(msg) = msg {
            match msg {
                Action::Exit => {
                    self.token.cancel();
                    self.exit = true;
                }
                Action::MoveUp => {
                    self.move_up();
                }
                Action::MoveDown => {
                    self.move_down();
                }
                Action::MoveLeft => {
                    self.move_left();
                }
                Action::MoveRight => {
                    self.move_right();
                }
                Action::Enter => {
                    self.move_down();
                    self.selected_letter = 0;
                }
                Action::EnterChar(x) => {
                    let res = self.set_letter(Some(x));
                    self.action_tx.send(res).unwrap();
                    self.move_right();
                }
                Action::DeleteChar => {
                    let res = self.set_letter(None);
                    self.action_tx.send(res).unwrap();
                    self.move_left();
                }
                Action::ToggleStatus => {
                    let res = self.toggle_status();
                    self.action_tx.send(res).unwrap()
                }
                Action::UpdateGuesses => {
                    self.update_guesses();
                }
                Action::GetSuggestions(guesses) => {
                    let sovler = self.solver.clone();
                    let two_level = self.two_level;
                    let tx = self.action_tx.clone();

                    if let Some(token) = self.child_token.take() {
                        token.cancel();
                    }

                    let child = self.token.child_token();
                    let child_clone = child.clone();
                    self.child_token = Some(child.clone());

                    tokio::spawn(async move {
                        let suggestions = tokio::select! {
                            biased;
                            _ = child_clone.cancelled() => {
                                // The token was cancelled
                                None
                            }
                            x = get_suggestions(&sovler, guesses, two_level) => {
                                Some(x)
                            }
                        };
                        if !child.is_cancelled() {
                            if let Some(suggestions) = suggestions {
                                tx.send(Some(Action::UpdateSuggestions(suggestions)))
                                    .unwrap();
                            }
                        }
                    });
                }
                Action::UpdateSuggestions(suggestions) => {
                    self.suggestions = suggestions;
                }
            }
        }
    }

    fn move_right(&mut self) {
        if self.selected_letter < 4 {
            self.selected_letter += 1;
        }
    }

    fn move_left(&mut self) {
        if self.selected_letter > 0 {
            self.selected_letter -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.selected_word < self.guesses.len() - 1 {
            self.selected_word += 1;
        }
    }

    fn move_up(&mut self) {
        if self.selected_word > 0 {
            self.selected_word -= 1;
        }
    }

    fn set_letter(&mut self, letter: Option<char>) -> Option<Action> {
        self.guesses[self.selected_word].set_letter(letter, self.selected_letter);
        if letter.is_none() {
            self.guesses[self.selected_word]
                .update_status(LetterStatus::Absent, self.selected_letter)
        }
        // self.update_guesses();
        Some(Action::UpdateGuesses)
    }

    fn toggle_status(&mut self) -> Option<Action> {
        if self.guesses[self.selected_word].word.chars[self.selected_letter].is_some() {
            use LetterStatus::*;
            let current =
                decode_status(self.guesses[self.selected_word].status)[self.selected_letter];
            let new = match current {
                Absent => Misplaced,
                Misplaced => Correct,
                Correct => Absent,
            };
            self.guesses[self.selected_word].update_status(new, self.selected_letter);
            Some(Action::UpdateGuesses)
        } else {
            None
        }
    }

    fn update_guesses(&mut self) {
        let mut tmp = [Guess::empty(); 6];

        for (i, item) in tmp.iter_mut().enumerate() {
            let current_guess = self.guesses[i];
            if self.solver.is_valid_guess(&current_guess.word) {
                *item = current_guess
            } else {
                break;
            }
        }

        if tmp != self.cached_guesses {
            self.cached_guesses = tmp;
            let tmp: Vec<Guess> = tmp
                .into_iter()
                .filter(|guess| guess.word.chars.iter().all(|c| c.is_some()))
                .collect();
            self.action_tx
                .send(Some(Action::GetSuggestions(tmp.clone())))
                .unwrap();
            self.remaining_words = self.solver.get_remaining_words_idx(&tmp);
            // self.update_solutions(&tmp);
            self.update_evaluations(&tmp);
        }
    }

    fn update_evaluations(&mut self, guesses: &[Guess]) {
        let mut eva: Vec<GuessEvaluation> = vec![];

        for (i, g) in guesses.iter().enumerate() {
            let remaining_words = self.solver.get_remaining_words_idx(&guesses[0..i]);
            let e = self.solver.evalute_guess(
                &g.word,
                &remaining_words,
                Some(decode_status(g.status)),
                false,
            );
            eva.push(e)
        }
        self.evaludations = eva;
    }
}

async fn get_suggestions(
    solver: &Solver,
    guesses: Vec<Guess>,
    two_level: bool,
) -> Vec<GuessEvaluation> {
    let remaining_words = solver.get_remaining_words_idx(&guesses);

    let penalty = if guesses.is_empty() { 0.0 } else { 0.1 };

    let suggestions: Vec<GuessEvaluation> = solver
        .guess(N_SUGGESTIONS, &remaining_words, penalty)
        .iter()
        .map(|w| solver.evalute_guess(w, &remaining_words, None, two_level))
        .collect();
    suggestions
}
