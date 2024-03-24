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
