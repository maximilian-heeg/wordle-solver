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

    37 words could not be solved in 6 guesses:   doozy, dozed, goxes,
       javas, jests, jills, jongs, kacks, kents, kinks, lills, mongs,
       nexts, pents, sades, sangs, saxes, sings, soles, songs, vants,
       vells, vents, vests, veves, vexes, vills, vines, waxes, wexes,
       wonks, wulls, yayas, zests, zexes, zezes, zills

    Out of these, all expect for `zills` can be solved in 7 steps. `zills` requires 8 attempts.

    The others have been solved in an average of 4.11 steps
    Here are the numbers for how many wordles have been solved in n steps.
    Steps 1: Count 1
    Steps 2: Count 52
    Steps 3: Count 2733
    Steps 4: Count 7963
    Steps 5: Count 3613
    Steps 6: Count 456
