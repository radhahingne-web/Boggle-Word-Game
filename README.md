# word_games – Spelling Bee and Boggle in Rust

Two terminal word games written in beginner-friendly Rust.
No outside crates are used, only Rust's standard library.

## Folder layout

```
word_games/
├── Cargo.toml
├── words.txt                  <- the dictionary (one lowercase word per line)
└── src/
    ├── lib.rs                 <- lists the modules below
    ├── rng.rs                 <- tiny random number generator
    ├── words.rs               <- loads the dictionary file
    ├── trie.rs                <- trie data structure (fast word lookup)
    ├── console.rs             <- reads a line typed by the player
    ├── spelling_bee.rs        <- Spelling Bee rules, scoring, hints   (+ tests)
    ├── boggle.rs              <- Boggle grid, word search, scoring    (+ tests)
    └── bin/
        ├── play_spelling_bee.rs   <- the Spelling Bee program (screen + keyboard)
        └── play_boggle.rs         <- the Boggle program (screen + keyboard)
```

The "rules" (`spelling_bee.rs`, `boggle.rs`) never print anything, and the
"screen" programs (`bin/`) contain no rules. This makes the rules easy to test.

## Cargo commands

Run these from the `word_games` folder (the one containing `Cargo.toml` and `words.txt`).

| What you want                        | Command                                      |
|--------------------------------------|----------------------------------------------|
| Check for errors quickly             | `cargo check`                                |
| Build                                | `cargo build`                                |
| Run all unit tests                   | `cargo test`                                 |
| Play Spelling Bee                    | `cargo run --release --bin play_spelling_bee`|
| Play Boggle                          | `cargo run --release --bin play_boggle`      |
| Build the fast "release" versions    | `cargo build --release`                      |
| Make the HTML documentation          | `cargo doc --open`                           |

`--release` makes the games start much faster (loading 370,000 words).
The release programs end up in `target/release/`. Run them from the folder
that contains `words.txt`.

## How to play

**Spelling Bee** – you get 7 letters; the first is the main letter. Find words
of 4+ letters that use only those letters and always include the main letter.
4-letter words score 1 point, longer words 1 point per letter, and a pangram
(uses all 7 letters) earns 7 bonus points.
Commands: `:hint` (hint grid + two-letter list), `:help`, `:quit`.

**Boggle** – choose a 4x4, 5x5 or 6x6 grid. Find words (3+ letters) by moving
to touching squares (including diagonals) without reusing a square.
Type words separated by spaces, commas or new lines. Nothing is checked until
you type `:done`; wrong guesses cost 1 point. "Q" always appears as "QU".
Commands: `:done`, `:grid`, `:help`, `:quit`.
Scores: 3-4 letters = 1, 5 = 2, 6 = 3, 7 = 5, 8 = 11, 9+ = 2 per letter.

Commands start with a colon so they can never be confused with real words
(`done` and `quit` are real English words!).

## The dictionary

`words.txt` has about 370,000 words (from the public-domain `words_alpha.txt`
list of the dwyl/english-words project). You can replace it with any text file
with one word per line. Lines that contain capital letters, hyphens, digits or
apostrophes are skipped automatically.

## Testing notes (meta-documentation)

Run `cargo test`. There are 40 unit tests; each sits next to the code it tests,
in a `#[cfg(test)] mod tests` block at the bottom of the file.

| Module         | What the tests check |
|----------------|----------------------|
| `rng`          | same seed -> same numbers; `below(n)` stays in range; shuffling keeps the same letters |
| `words`        | capitals / hyphens / apostrophes are dropped; list is sorted and de-duplicated; missing file gives an error |
| `trie`         | inserted words are found; other words and prefixes are not |
| `spelling_bee` | a tiny dictionary where the answers were worked out **by hand**: correct answer list, scoring (4-letter, longer, pangram), each kind of bad guess gives the right `GuessResult`, duplicates, alphabetical order, Queen Bee, hint grid and two-letter list |
| `boggle`       | a 3x3 grid we know by heart: rows, columns, diagonals, backwards words, no square reused, "Qu" squares, scoring table, input splitting and format checks, whole-guess evaluation |

**Random testing.** `Rng::from_seed` gives repeatable "random" numbers, so a
failing random test can be reproduced. Tests loop over many seeds to check that:
the 7 Spelling Bee letters are always distinct; random Boggle grids only contain
lowercase letters or "qu"; and a word read along a path of a random grid is
always found by the search.

**Manual / system testing** (done while building):
* Played both games with scripted input, trying empty lines, too-short words,
  words with outside letters, missing main letter, repeated words, unknown
  commands, bad grid sizes and a missing dictionary file.
* Cross-checked the Boggle search against a separate Python solver on random
  grids using the full dictionary: both accepted exactly the same words.
* Won Spelling Bee ("Queen Bee") with a tiny dictionary to test that ending.
