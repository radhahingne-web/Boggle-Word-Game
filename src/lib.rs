//! # word_games
//!
//! A small library with everything the two games need.
//! Each module has one job (this is called "encapsulation"):
//!
//! * `rng`          - a tiny random number generator
//! * `words`        - loading the dictionary file
//! * `trie`         - a fast word lookup structure (used by Boggle)
//! * `console`      - reading a line typed by the player
//! * `spelling_bee` - the rules and scoring of Spelling Bee
//! * `boggle`       - the rules and scoring of Boggle
//!
//! The two programs you can run live in `src/bin/`:
//! `play_spelling_bee` and `play_boggle`.

pub mod boggle;
pub mod console;
pub mod rng;
pub mod spelling_bee;
pub mod trie;
pub mod words;
