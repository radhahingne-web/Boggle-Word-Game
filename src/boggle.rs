//! Boggle: the grid, the rules for finding a word in it, and the scoring.
//!
//! Like `spelling_bee.rs`, nothing here prints or reads from the keyboard.
//! That is done in `src/bin/play_boggle.rs`.
//!
//! The rules:
//! * A word needs at least 3 letters.
//! * After the first letter, every next letter must touch the one before it
//!   (up, down, left, right or diagonal).
//! * A square on the grid may not be used twice in the same word.
//! * The word must be in the dictionary.
//!
//! `Q` always appears on the grid as "Qu" (two letters in ONE square).
//! A word using that square gets credit for both letters.

use crate::rng::Rng;
use crate::trie::Trie;

// ---------- Constants ----------

/// The shortest word the player may guess.
pub const MIN_WORD_LENGTH: usize = 3;

/// Points taken away for each guess that is not a word (or is not on the grid).
pub const WRONG_GUESS_PENALTY: i32 = 1;

/// How common each letter is when we make a random grid
/// (the same counts as the tiles in the game Scrabble).
/// An array holds a fixed number of items: here 26 pairs of (letter, count).
const LETTER_COUNTS: [(char, usize); 26] = [
    ('a', 9), ('b', 2), ('c', 2), ('d', 4), ('e', 12), ('f', 2), ('g', 3),
    ('h', 2), ('i', 9), ('j', 1), ('k', 1), ('l', 4), ('m', 2), ('n', 6),
    ('o', 8), ('p', 2), ('q', 1), ('r', 6), ('s', 4), ('t', 6), ('u', 4),
    ('v', 2), ('w', 2), ('x', 1), ('y', 2), ('z', 1),
];

// ---------- The grid ----------

/// A square grid of letters.
pub struct Board {
    size: usize,
    /// cells[row][column] holds the text of one square: "a", "t", "qu", ...
    cells: Vec<Vec<String>>,
}

impl Board {
    /// Make a random `size` x `size` grid.
    pub fn random(size: usize, rng: &mut Rng) -> Board {
        // Build a "bag" of letters where common letters appear more often.
        let mut bag: Vec<char> = Vec::new();
        for (letter, count) in LETTER_COUNTS.iter() {
            for _ in 0..*count {
                bag.push(*letter);
            }
        }

        // Fill every square with a random letter from the bag.
        let mut cells = Vec::new();
        for _row in 0..size {
            let mut row_cells = Vec::new();
            for _col in 0..size {
                let letter = bag[rng.below(bag.len())];
                if letter == 'q' {
                    row_cells.push("qu".to_string()); // Q always comes with U
                } else {
                    row_cells.push(letter.to_string());
                }
            }
            cells.push(row_cells);
        }
        Board { size, cells }
    }

    /// Make a grid from rows of text, one character per square.
    /// Handy for tests, where we want to know exactly what is on the grid.
    /// Example: `Board::from_rows(&["cat", "dog", "xyz"])`
    pub fn from_rows(rows: &[&str]) -> Board {
        let mut cells = Vec::new();
        for row in rows {
            let mut row_cells = Vec::new();
            for letter in row.chars() {
                if letter == 'q' {
                    row_cells.push("qu".to_string());
                } else {
                    row_cells.push(letter.to_string());
                }
            }
            cells.push(row_cells);
        }
        Board {
            size: rows.len(),
            cells,
        }
    }

    /// The grid is `size` squares wide and `size` squares tall.
    pub fn size(&self) -> usize {
        self.size
    }

    /// The text in one square.
    pub fn cell(&self, row: usize, col: usize) -> &str {
        &self.cells[row][col]
    }

    /// Can `word` be traced out on the grid following the neighbour rules?
    /// We try every square as the starting point.
    pub fn can_make_word(&self, word: &str) -> bool {
        // used[row][col] is true while a square is part of the path we are building.
        let mut used = vec![vec![false; self.size]; self.size];
        for row in 0..self.size {
            for col in 0..self.size {
                if self.search(row, col, word, 0, &mut used) {
                    return true;
                }
            }
        }
        false
    }

    /// Helper for `can_make_word`. It asks:
    /// "Can the part of `word` starting at byte `position` be matched,
    ///  beginning on the square (row, col)?"
    ///
    /// This is a *recursive* function: it calls itself for each neighbour.
    /// If a path turns out to be a dead end we "backtrack" (undo the step)
    /// and try another way.
    fn search(
        &self,
        row: usize,
        col: usize,
        word: &str,
        position: usize,
        used: &mut Vec<Vec<bool>>, // `&mut` = we borrow it AND may change it
    ) -> bool {
        let square = self.cells[row][col].as_str();
        let rest_of_word = &word[position..];

        // Does this square match the next letter(s) of the word?
        if !rest_of_word.starts_with(square) {
            return false;
        }

        // Yes. Did we just match the end of the word? Then we found it!
        let next_position = position + square.len();
        if next_position == word.len() {
            return true;
        }

        // Otherwise, mark this square as used and try each neighbour.
        used[row][col] = true;
        for row_step in [-1i32, 0, 1] {
            for col_step in [-1i32, 0, 1] {
                if row_step == 0 && col_step == 0 {
                    continue; // that is the square we are standing on
                }
                let new_row = row as i32 + row_step;
                let new_col = col as i32 + col_step;

                // Stay inside the grid.
                if new_row < 0 || new_col < 0 {
                    continue;
                }
                let new_row = new_row as usize;
                let new_col = new_col as usize;
                if new_row >= self.size || new_col >= self.size {
                    continue;
                }

                // Never use the same square twice in one word.
                if used[new_row][new_col] {
                    continue;
                }

                if self.search(new_row, new_col, word, next_position, used) {
                    used[row][col] = false; // tidy up before leaving
                    return true;
                }
            }
        }
        used[row][col] = false; // backtrack: this square is free again
        false
    }
}

// ---------- Reading what the player typed ----------

/// Split one typed line into separate words.
/// Words may be separated by commas, spaces, or both: "cat, dog  bird".
pub fn split_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for piece in line.split(|c: char| c == ',' || c.is_whitespace()) {
        if !piece.is_empty() {
            tokens.push(piece.to_string());
        }
    }
    tokens
}

/// Check that a typed word is well formed.
/// On success we get the word back in lowercase; otherwise a polite message.
pub fn check_format(token: &str) -> Result<String, String> {
    if !token.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(format!(
            "'{}' was skipped: words may only contain the letters a-z \
             (no digits, hyphens or apostrophes).",
            token
        ));
    }
    if token.len() < MIN_WORD_LENGTH {
        return Err(format!(
            "'{}' was skipped: words need at least {} letters.",
            token, MIN_WORD_LENGTH
        ));
    }
    Ok(token.to_lowercase())
}

// ---------- Scoring ----------

/// How many points is a word worth?
/// 3-4 letters: 1   5: 2   6: 3   7: 5   8: 11   9 or more: 2 per letter.
/// (A "Qu" square counts as two letters.)
pub fn word_points(word: &str) -> u32 {
    let length = word.len() as u32;
    match length {
        0..=2 => 0,
        3 | 4 => 1,
        5 => 2,
        6 => 3,
        7 => 5,
        8 => 11,
        _ => 2 * length,
    }
}

/// Why a guess did not count.
#[derive(Debug, PartialEq)]
pub enum Rejection {
    NotInDictionary,
    NotOnGrid,
}

/// The result of checking all of the player's guesses.
pub struct Report {
    /// Good words and the points each one earned.
    pub accepted: Vec<(String, u32)>,
    /// Bad words and the reason each was rejected.
    pub rejected: Vec<(String, Rejection)>,
    /// Final score (can be negative if the player guessed badly!).
    pub score: i32,
}

/// Check every guess against the dictionary and the grid, and add up the score.
pub fn evaluate(board: &Board, dictionary: &Trie, guesses: &[String]) -> Report {
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    let mut score: i32 = 0;

    for guess in guesses {
        if !dictionary.contains(guess) {
            rejected.push((guess.clone(), Rejection::NotInDictionary));
            score -= WRONG_GUESS_PENALTY;
        } else if !board.can_make_word(guess) {
            rejected.push((guess.clone(), Rejection::NotOnGrid));
            score -= WRONG_GUESS_PENALTY;
        } else {
            let points = word_points(guess);
            accepted.push((guess.clone(), points));
            score += points as i32;
        }
    }

    Report {
        accepted,
        rejected,
        score,
    }
}

// ---------- Tests (run them with `cargo test`) ----------

#[cfg(test)]
mod tests {
    use super::*;

    /// A grid we know by heart:
    ///   c a t
    ///   d o g
    ///   x y z
    fn test_board() -> Board {
        Board::from_rows(&["cat", "dog", "xyz"])
    }

    #[test]
    fn finds_words_along_rows_and_diagonals() {
        let board = test_board();
        assert!(board.can_make_word("cat")); // along the top row
        assert!(board.can_make_word("dog")); // along the middle row
        assert!(board.can_make_word("cog")); // c -> o is a diagonal step
        assert!(board.can_make_word("cot")); // o -> t is a diagonal step
        assert!(board.can_make_word("cad")); // a -> d is a diagonal step
    }

    #[test]
    fn finds_words_going_backwards_and_up() {
        let board = test_board();
        assert!(board.can_make_word("tac")); // right to left
        assert!(board.can_make_word("god")); // right to left
        assert!(board.can_make_word("doc")); // d -> o -> c goes up-left
    }

    #[test]
    fn rejects_letters_that_are_not_neighbours() {
        let board = test_board();
        assert!(!board.can_make_word("ctx")); // c and t are two squares apart
        assert!(!board.can_make_word("catx")); // t and x are far apart
    }

    #[test]
    fn cannot_reuse_a_square() {
        let board = test_board();
        assert!(!board.can_make_word("tat")); // there is only one 't'
        assert!(!board.can_make_word("coco")); // only one 'c' and one 'o'
    }

    #[test]
    fn rejects_letters_that_are_not_on_the_grid() {
        let board = test_board();
        assert!(!board.can_make_word("bat"));
    }

    #[test]
    fn qu_is_one_square_with_two_letters() {
        // qu a
        // t  b
        let board = Board::from_rows(&["qa", "tb"]);
        assert_eq!(board.cell(0, 0), "qu");
        assert!(board.can_make_word("quat")); // qu -> a -> t
        assert!(board.can_make_word("quab")); // qu -> a -> b
        assert!(!board.can_make_word("qat")); // the grid has "qu", not "q"
    }

    #[test]
    fn works_for_every_grid_size() {
        for size in [4, 5, 6] {
            let mut rng = Rng::from_seed(size as u64);
            let board = Board::random(size, &mut rng);
            assert_eq!(board.size(), size);
            // Every square must hold at least one letter.
            for row in 0..size {
                for col in 0..size {
                    assert!(!board.cell(row, col).is_empty());
                }
            }
        }
    }

    #[test]
    fn random_grids_only_hold_lowercase_letters_and_qu() {
        // Test many random grids (the "random testing" idea).
        for seed in 1..=100 {
            let mut rng = Rng::from_seed(seed);
            let board = Board::random(4, &mut rng);
            for row in 0..4 {
                for col in 0..4 {
                    let cell = board.cell(row, col);
                    let ok = cell == "qu" || (cell.len() == 1 && cell.chars().all(|c| c.is_ascii_lowercase()));
                    assert!(ok, "bad square: {}", cell);
                }
            }
        }
    }

    #[test]
    fn a_word_read_off_a_random_grid_is_always_found() {
        // Take a real path across a random grid (left to right along the top row),
        // write down the letters, and check the program agrees it is on the grid.
        for seed in 1..=50 {
            let mut rng = Rng::from_seed(seed);
            let board = Board::random(5, &mut rng);
            let mut word = String::new();
            for col in 0..5 {
                word.push_str(board.cell(0, col));
            }
            assert!(board.can_make_word(&word), "missed the word {}", word);
        }
    }

    #[test]
    fn scoring_table_is_followed() {
        assert_eq!(word_points("cat"), 1);
        assert_eq!(word_points("cats"), 1);
        assert_eq!(word_points("catch"), 2);
        assert_eq!(word_points("catchy"), 3);
        assert_eq!(word_points("catches"), 5);
        assert_eq!(word_points("catching"), 11);
        assert_eq!(word_points("catchings"), 18); // 9 letters x 2
        assert_eq!(word_points("catchingly"), 20); // 10 letters x 2
    }

    #[test]
    fn splitting_handles_commas_spaces_and_blanks() {
        assert_eq!(split_line("cat dog"), vec!["cat", "dog"]);
        assert_eq!(split_line("cat,dog"), vec!["cat", "dog"]);
        assert_eq!(split_line("  cat ,  dog,, bird "), vec!["cat", "dog", "bird"]);
        assert!(split_line("   ").is_empty());
    }

    #[test]
    fn format_check_accepts_good_words_and_lowercases_them() {
        assert_eq!(check_format("Cat"), Ok("cat".to_string()));
    }

    #[test]
    fn format_check_rejects_bad_words_politely() {
        assert!(check_format("to").is_err()); // too short
        assert!(check_format("well-known").is_err()); // hyphen
        assert!(check_format("don't").is_err()); // apostrophe
        assert!(check_format("abc123").is_err()); // digits
    }

    #[test]
    fn evaluate_scores_good_words_and_penalises_bad_ones() {
        let board = test_board();
        let dictionary = Trie::from_words(&[
            "cat".to_string(),
            "dog".to_string(),
            "cot".to_string(),
            "bat".to_string(), // a real word, but there is no 'b' on the grid
        ]);
        let guesses: Vec<String> = ["cat", "dog", "cot", "bat", "xyzzy"]
            .iter()
            .map(|w| w.to_string())
            .collect();

        let report = evaluate(&board, &dictionary, &guesses);

        assert_eq!(report.accepted.len(), 3); // cat, dog, cot = 3 points
        assert_eq!(
            report.rejected,
            vec![
                ("bat".to_string(), Rejection::NotOnGrid),
                ("xyzzy".to_string(), Rejection::NotInDictionary),
            ]
        );
        assert_eq!(report.score, 3 - 2); // 3 points earned, 2 penalties
    }
}
