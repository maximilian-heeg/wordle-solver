## Wordle solver

### Run

- `cargo run --release` for TUI
- `cargo run --release benchmark` for testing all words in the dictionary
- `cargo run --release solve water` get the steps to sovle for the word "water"
- `cargo run --release -- -m 4 solve water` get the steps to sovle for the word "water". limit to 4 steps

### Commands

| Key                 | Command                            |
| ------------------- | ---------------------------------- |
| `a-z`               | Insert letter at selected position |
| `DEL`               | Delete letter at selected position |
| `1`                 | Add a new word                     |
| `9`                 | Remove last guess                  |
| `TAB`               | Toggle status of letter            |
| `PageUp` `PageDown` | Scroll through possible solutions  |
| `ArrowKeys`         | Select letter                      |

### Key status codes

| Status   |                                 |
| -------- | ------------------------------- |
| `grey`   | Unkown (letter will be ignored) |
| `red`    | Incorrect letter                |
| `yellow` | Misplaced letter                |
| `green`  | correct letter                  |

### Benchmark

Results of `cargo run --release benchmark`:

    0 words could not be solved in 6 guesses:
    The others have been solved in an average of 3.57 steps
    Here are the numbers for how many wordles have been solved in n steps.
    Steps 2: Count 69
    Steps 3: Count 1414
    Steps 4: Count 1521
    Steps 5: Count 178
    Steps 6: Count 7
