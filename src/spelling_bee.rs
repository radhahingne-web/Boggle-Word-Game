//! Spelling Bee: the rules, the word list and the scoring.
//!
//! Nothing in this file prints or reads from the keyboard. That is done in
//! `src/bin/play_spelling_bee.rs`. Keeping the "rules" separate from the
//! "screen" makes the rules easy to test.
//!
//! The rules:
//! * There are 7 different letters. The FIRST one is the main letter.
//! * A word needs at least 4 letters, must use the main letter, and may only
//!   use the 7 letters (each one can be used many times).
//! * A word scores 1 point if it has 4 letters, otherwise 1 point per letter.
//! * A PANGRAM (uses all 7 letters) gets 7 bonus points.

use crate::rng::Rng;
use std::collections::BTreeMap;

// ---------- Constants (values that never change) ----------

/// The shortest word the player may guess.
pub const MIN_WORD_LENGTH: usize = 4;
/// How many letters are in a puzzle.
pub const NUM_LETTERS: usize = 7;
/// Extra points for a pangram.
pub const PANGRAM_BONUS: u32 = 7;
/// We try to make puzzles with between 20 and 100 answers:
/// not too boring, and not so many that nobody could finish.
const MIN_ANSWERS_WANTED: usize = 20;
const MAX_ANSWERS_WANTED: usize = 100;
/// ...but we only try this many times before giving up and using what we have
/// (a very small dictionary may not be able to make a puzzle of that size).
const MAX_TRIES: usize = 200;

// ---------- The result of a guess ----------

/// Everything that can happen when the player makes a guess.
/// An `enum` is a type that can be exactly ONE of several choices.
#[derive(Debug, PartialEq)]
pub enum GuessResult {
    /// The word was right! Says how many points it earned.
    Correct { points: u32, is_pangram: bool },
    /// Fewer than 4 letters.
    TooShort,
    /// Uses letters that are not in the puzzle (the list says which ones).
    BadLetters(Vec<char>),
    /// Does not contain the main letter.
    MissingMainLetter,
    /// The player already found this word.
    AlreadyGuessed,
    /// A fine guess, but it is not in our word list.
    NotInList,
}

// ---------- Helper functions ----------

/// The different letters in a word, in the order they first appear.
/// "hello" gives ['h', 'e', 'l', 'o'].
fn distinct_letters(word: &str) -> Vec<char> {
    let mut letters = Vec::new();
    for c in word.chars() {
        if !letters.contains(&c) {
            letters.push(c);
        }
    }
    letters
}

/// Does this word use all of the puzzle's letters?
pub fn is_pangram(word: &str, letters: &[char]) -> bool {
    letters.iter().all(|letter| word.contains(*letter))
}

/// How many points is a word worth?
pub fn word_points(word: &str, letters: &[char]) -> u32 {
    let length = word.len() as u32;
    let mut points = if length == 4 { 1 } else { length };
    if is_pangram(word, letters) {
        points += PANGRAM_BONUS;
    }
    points
}

/// Look through the whole dictionary and collect every word that can be
/// played: at least 4 letters, contains the main letter (`letters[0]`),
/// and uses only the puzzle's letters.
/// We only borrow `letters` and `dictionary` (`&`), we do not take them over.
pub fn find_answers(letters: &[char], dictionary: &[String]) -> Vec<String> {
    let main_letter = letters[0];
    let mut answers = Vec::new();
    for word in dictionary {
        if word.len() < MIN_WORD_LENGTH {
            continue; // too short, skip to the next word
        }
        if !word.contains(main_letter) {
            continue; // main letter missing
        }
        if word.chars().all(|c| letters.contains(&c)) {
            answers.push(word.clone());
        }
    }
    answers.sort();
    answers
}

/// Choose 7 different letters for a new puzzle. The main letter comes first.
///
/// Trick: we start from a real word that has exactly 7 different letters.
/// That way the puzzle always has at least one pangram!
/// Returns `None` if the dictionary has no such word.
pub fn generate_letters(dictionary: &[String], rng: &mut Rng) -> Option<Vec<char>> {
    // Step 1: collect words that have exactly 7 different letters.
    let mut seed_words: Vec<&String> = Vec::new();
    for word in dictionary {
        if distinct_letters(word).len() == NUM_LETTERS {
            seed_words.push(word);
        }
    }
    if seed_words.is_empty() {
        return None;
    }

    // Step 2: pick one at random, shuffle its letters, and check the puzzle
    // has a nice number of answers. Try again if it does not.
    let mut letters = Vec::new();
    for _ in 0..MAX_TRIES {
        let word = seed_words[rng.below(seed_words.len())];
        letters = distinct_letters(word);
        rng.shuffle(&mut letters);
        let number_of_answers = find_answers(&letters, dictionary).len();
        if number_of_answers >= MIN_ANSWERS_WANTED && number_of_answers <= MAX_ANSWERS_WANTED {
            break;
        }
    }
    Some(letters) // after shuffling, the first letter is the main letter
}

// ---------- The puzzle itself ----------

/// One game of Spelling Bee.
/// The fields are private: other code must use the methods below.
pub struct Puzzle {
    letters: Vec<char>,    // letters[0] is the main letter
    answers: Vec<String>,  // every playable word (sorted)
    guessed: Vec<String>,  // what the player found so far (kept sorted)
    score: u32,
}

impl Puzzle {
    /// Start a new puzzle.
    /// `letters` is MOVED into the puzzle (the puzzle now owns it),
    /// while `dictionary` is only borrowed.
    pub fn new(letters: Vec<char>, dictionary: &[String]) -> Puzzle {
        let answers = find_answers(&letters, dictionary);
        Puzzle {
            letters,
            answers,
            guessed: Vec::new(),
            score: 0,
        }
    }

    // --- Simple "getters": let outside code look, but not change ---

    pub fn letters(&self) -> &Vec<char> {
        &self.letters
    }

    pub fn main_letter(&self) -> char {
        self.letters[0]
    }

    pub fn guessed(&self) -> &Vec<String> {
        &self.guessed
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn answer_count(&self) -> usize {
        self.answers.len()
    }

    /// Total points available if the player found every word.
    pub fn total_points(&self) -> u32 {
        let mut total = 0;
        for word in &self.answers {
            total += word_points(word, &self.letters);
        }
        total
    }

    /// How many pangrams are hiding in the puzzle?
    pub fn pangram_count(&self) -> usize {
        self.answers
            .iter()
            .filter(|word| is_pangram(word, &self.letters))
            .count()
    }

    /// Has the player found every word? ("Queen Bee!")
    pub fn is_complete(&self) -> bool {
        !self.answers.is_empty() && self.guessed.len() == self.answers.len()
    }

    /// The words the player has not found yet.
    pub fn missed_words(&self) -> Vec<String> {
        let mut missed = Vec::new();
        for word in &self.answers {
            if !self.guessed.contains(word) {
                missed.push(word.clone());
            }
        }
        missed
    }

    // --- Making a guess ---

    /// Check a guess. If it is correct, add its points and remember it.
    /// `&mut self` means this method is allowed to change the puzzle.
    pub fn guess(&mut self, text: &str) -> GuessResult {
        let word = text.trim().to_lowercase();

        if word.chars().count() < MIN_WORD_LENGTH {
            return GuessResult::TooShort;
        }

        // Collect letters that are not allowed (without repeating any).
        let mut bad_letters = Vec::new();
        for c in word.chars() {
            if !self.letters.contains(&c) && !bad_letters.contains(&c) {
                bad_letters.push(c);
            }
        }
        if !bad_letters.is_empty() {
            return GuessResult::BadLetters(bad_letters);
        }

        if !word.contains(self.main_letter()) {
            return GuessResult::MissingMainLetter;
        }
        if self.guessed.contains(&word) {
            return GuessResult::AlreadyGuessed;
        }
        if !self.answers.contains(&word) {
            return GuessResult::NotInList;
        }

        // It is a good word!
        let points = word_points(&word, &self.letters);
        let pangram = is_pangram(&word, &self.letters);
        self.score += points;
        self.guessed.push(word);
        self.guessed.sort(); // keep the list in alphabetical order
        GuessResult::Correct {
            points,
            is_pangram: pangram,
        }
    }

    // --- Hints ---

    /// The longest answer's length.
    pub fn longest_answer(&self) -> usize {
        self.answers
            .iter()
            .map(|word| word.len())
            .max()
            .unwrap_or(MIN_WORD_LENGTH)
    }

    /// The hint grid: for each starting letter, how many answers of each
    /// length (4, 5, 6, ... up to the longest) begin with it.
    /// Letters that start no words are left out.
    pub fn hint_grid(&self) -> Vec<(char, Vec<usize>)> {
        let number_of_lengths = self.longest_answer() - MIN_WORD_LENGTH + 1;

        let mut sorted_letters = self.letters.clone(); // a copy we can sort
        sorted_letters.sort();

        let mut rows = Vec::new();
        for letter in sorted_letters {
            let mut counts = vec![0; number_of_lengths];
            for word in &self.answers {
                if word.starts_with(letter) {
                    counts[word.len() - MIN_WORD_LENGTH] += 1;
                }
            }
            if counts.iter().sum::<usize>() > 0 {
                rows.push((letter, counts));
            }
        }
        rows
    }

    /// The two-letter list: how many answers start with each pair of letters.
    /// A `BTreeMap` keeps its keys in alphabetical order for us.
    pub fn two_letter_counts(&self) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();
        for word in &self.answers {
            let start = word[0..2].to_string();
            *counts.entry(start).or_insert(0) += 1;
        }
        counts
    }
}

// ---------- Tests (run them with `cargo test`) ----------

#[cfg(test)]
mod tests {
    use super::*;

    const LETTERS: [char; 7] = ['c', 'a', 'i', 'm', 'o', 'r', 't']; // main letter: c

    /// A tiny sorted dictionary where we know the right answers by hand.
    fn small_dictionary() -> Vec<String> {
        let words = [
            "acid",     // has 'd'               -> not allowed
            "aromatic", // pangram               -> answer (15 points)
            "atom",     // no main letter 'c'    -> not an answer
            "atomic",   //                       -> answer (6)
            "cat",      // too short             -> not an answer
            "citric",   //                       -> answer (6)
            "coma",     //                       -> answer (1)
            "comic",    //                       -> answer (5)
            "mart",     // no main letter 'c'    -> not an answer
            "tact",     //                       -> answer (1)
        ];
        words.iter().map(|w| w.to_string()).collect()
    }

    fn new_puzzle() -> Puzzle {
        Puzzle::new(LETTERS.to_vec(), &small_dictionary())
    }

    #[test]
    fn finds_exactly_the_right_answers() {
        let answers = find_answers(&LETTERS, &small_dictionary());
        assert_eq!(
            answers,
            vec!["aromatic", "atomic", "citric", "coma", "comic", "tact"]
        );
    }

    #[test]
    fn scoring_follows_the_rules() {
        assert_eq!(word_points("coma", &LETTERS), 1); // 4 letters = 1 point
        assert_eq!(word_points("comic", &LETTERS), 5); // 5 letters = 5 points
        assert_eq!(word_points("atomic", &LETTERS), 6);
        assert_eq!(word_points("aromatic", &LETTERS), 8 + 7); // pangram bonus
    }

    #[test]
    fn puzzle_totals_are_correct() {
        let puzzle = new_puzzle();
        assert_eq!(puzzle.answer_count(), 6);
        assert_eq!(puzzle.total_points(), 15 + 6 + 6 + 1 + 5 + 1);
        assert_eq!(puzzle.pangram_count(), 1);
    }

    #[test]
    fn a_good_guess_scores_and_is_remembered() {
        let mut puzzle = new_puzzle();
        assert_eq!(
            puzzle.guess("COMIC"), // capitals are fine
            GuessResult::Correct { points: 5, is_pangram: false }
        );
        assert_eq!(puzzle.score(), 5);
        assert_eq!(puzzle.guessed(), &vec!["comic".to_string()]);
    }

    #[test]
    fn pangram_is_recognised() {
        let mut puzzle = new_puzzle();
        assert_eq!(
            puzzle.guess("aromatic"),
            GuessResult::Correct { points: 15, is_pangram: true }
        );
    }

    #[test]
    fn bad_guesses_are_rejected_with_the_right_reason() {
        let mut puzzle = new_puzzle();
        assert_eq!(puzzle.guess("cat"), GuessResult::TooShort);
        assert_eq!(puzzle.guess("acid"), GuessResult::BadLetters(vec!['d']));
        assert_eq!(puzzle.guess("mart"), GuessResult::MissingMainLetter);
        assert_eq!(puzzle.guess("critic"), GuessResult::NotInList); // allowed letters, not in our list
        assert_eq!(puzzle.score(), 0); // nothing was scored
    }

    #[test]
    fn bad_letters_are_listed_once_each() {
        let mut puzzle = new_puzzle();
        assert_eq!(
            puzzle.guess("zzzzxc"),
            GuessResult::BadLetters(vec!['z', 'x'])
        );
    }

    #[test]
    fn same_word_cannot_score_twice() {
        let mut puzzle = new_puzzle();
        puzzle.guess("coma");
        assert_eq!(puzzle.guess("coma"), GuessResult::AlreadyGuessed);
        assert_eq!(puzzle.score(), 1);
    }

    #[test]
    fn guessed_words_stay_alphabetical() {
        let mut puzzle = new_puzzle();
        puzzle.guess("tact");
        puzzle.guess("coma");
        puzzle.guess("atomic");
        assert_eq!(puzzle.guessed(), &vec!["atomic", "coma", "tact"]);
    }

    #[test]
    fn finding_every_word_completes_the_puzzle() {
        let mut puzzle = new_puzzle();
        assert!(!puzzle.is_complete());
        for word in ["aromatic", "atomic", "citric", "coma", "comic", "tact"] {
            puzzle.guess(word);
        }
        assert!(puzzle.is_complete());
        assert!(puzzle.missed_words().is_empty());
        assert_eq!(puzzle.score(), puzzle.total_points());
    }

    #[test]
    fn missed_words_are_listed() {
        let mut puzzle = new_puzzle();
        puzzle.guess("coma");
        assert_eq!(
            puzzle.missed_words(),
            vec!["aromatic", "atomic", "citric", "comic", "tact"]
        );
    }

    #[test]
    fn hint_grid_counts_match_the_answers() {
        let puzzle = new_puzzle();
        let grid = puzzle.hint_grid();
        // Lengths run 4..=8, so each row has 5 numbers.
        // a: atomic(6), aromatic(8)  c: coma(4), comic(5), citric(6)  t: tact(4)
        assert_eq!(
            grid,
            vec![
                ('a', vec![0, 0, 1, 0, 1]),
                ('c', vec![1, 1, 1, 0, 0]),
                ('t', vec![1, 0, 0, 0, 0]),
            ]
        );
    }

    #[test]
    fn two_letter_list_is_correct() {
        let counts = new_puzzle().two_letter_counts();
        assert_eq!(counts["ar"], 1);
        assert_eq!(counts["at"], 1);
        assert_eq!(counts["ci"], 1);
        assert_eq!(counts["co"], 2);
        assert_eq!(counts["ta"], 1);
        assert_eq!(counts.len(), 5);
    }

    #[test]
    fn generated_letters_are_seven_different_letters() {
        // Use many different random seeds to test it over and over.
        let dictionary = small_dictionary();
        for seed in 1..=50 {
            let mut rng = Rng::from_seed(seed);
            let letters = generate_letters(&dictionary, &mut rng).unwrap();
            assert_eq!(letters.len(), NUM_LETTERS);
            assert_eq!(distinct_letters(&letters.iter().collect::<String>()).len(), 7);
            // The only 7-letter word is "aromatic", so these must be its letters.
            let mut sorted = letters.clone();
            sorted.sort();
            assert_eq!(sorted, vec!['a', 'c', 'i', 'm', 'o', 'r', 't']);
        }
    }

    #[test]
    fn no_seven_letter_word_means_no_puzzle() {
        let dictionary = vec!["cat".to_string(), "dog".to_string()];
        let mut rng = Rng::from_seed(1);
        assert_eq!(generate_letters(&dictionary, &mut rng), None);
    }
}
