use super::*;

impl App {
    pub fn update_guesses(&mut self) {
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
            self.update_solutions(&tmp);
            self.update_evaluations(&tmp);
        }
    }

    fn update_solutions(&mut self, guesses: &[Guess]) {
        self.remaining_words = self.solver.get_remaining_words_idx(guesses);

        let penalty = if guesses.is_empty() { 0.0 } else { 0.1 };

        self.suggestions = self
            .solver
            .guess(N_SUGGESTIONS, &self.remaining_words, penalty)
            .iter()
            .map(|w| self.solver.evalute_guess(w, &self.remaining_words, None))
            .collect();
    }

    fn update_evaluations(&mut self, guesses: &[Guess]) {
        let mut eva: Vec<GuessEvaluation> = vec![];

        for (i, g) in guesses.iter().enumerate() {
            let remaining_words = self.solver.get_remaining_words_idx(&guesses[0..i]);
            let e =
                self.solver
                    .evalute_guess(&g.word, &remaining_words, Some(decode_status(g.status)));
            eva.push(e)
        }
        self.evaludations = eva;
    }
}
