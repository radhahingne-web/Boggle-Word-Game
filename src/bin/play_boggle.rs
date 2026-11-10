//! Play Boggle in the terminal.
//!
//! This file only deals with talking to the player (printing and typing).
//! All the rules live in `src/boggle.rs`.
//!
//! Run it with:  cargo run --release --bin play_boggle

use word_games::boggle::{self, Board, Rejection, Report};
use word_games::console::prompt;
use word_games::rng::Rng;
use word_games::trie::Trie;
use word_games::words;

/// The dictionary file: one word per line.
const DICTIONARY_FILE: &str = "words.txt";

fn main() {
    println!("=====================================");
    println!("          Welcome to BOGGLE");
    println!("=====================================");
    println!("Loading the word list (this takes a moment)...");

    // Load the words from the file and put them into a trie.
    let word_list = match words::load_words(DICTIONARY_FILE) {
        Ok(list) => list,
        Err(message) => {
            println!("{}", message);
            return;
        }
    };
    let dictionary = Trie::from_words(&word_list);
    let mut rng = Rng::from_clock();
    let mut total_score: i32 = 0; // adds up over several rounds

    // ---------- OUTER LOOP: one round of the game per trip around ----------
    loop {
        // 1. Ask for a grid size (or quit).
        let size = match ask_grid_size() {
            Some(size) => size,
            None => break,
        };

        // 2. Make and show the grid.
        let board = Board::random(size, &mut rng);
        print_board(&board);

        // 3. Collect the player's guesses. `None` means they chose to quit.
        let guesses = match collect_guesses(&board) {
            Some(guesses) => guesses,
            None => break,
        };

        // 4. Check the guesses and show the results.
        let report = boggle::evaluate(&board, &dictionary, &guesses);
        print_report(&report);
        total_score += report.score;
        println!("Your total score for all rounds: {}", total_score);

        // 5. Another round?
        if !ask_play_again() {
            break;
        }
    }

    println!();
    println!("Thanks for playing! Your final total score: {}", total_score);
}

/// Ask which grid size the player wants.
/// Returns `None` if they type :quit (or the input ends).
fn ask_grid_size() -> Option<usize> {
    loop {
        println!();
        let answer = prompt("Choose a grid size: 4, 5 or 6 (or :quit to leave): ")?;
        // The `?` above means: if the input ended (None), stop here and return None too.

        if answer.to_lowercase() == ":quit" {
            return None;
        }
        match answer.parse::<usize>() {
            Ok(size) if (4..=6).contains(&size) => return Some(size),
            _ => println!("Sorry, '{}' is not a size I can use. Please type 4, 5 or 6.", answer),
        }
    }
}

/// Ask "Play again?" until the player gives a clear yes or no.
fn ask_play_again() -> bool {
    loop {
        let answer = match prompt("\nPlay another round? (y/n): ") {
            Some(text) => text.to_lowercase(),
            None => return false,
        };
        if answer == "y" || answer == "yes" {
            return true;
        }
        if answer == "n" || answer == "no" || answer == ":quit" {
            return false;
        }
        println!("Please type y for yes or n for no.");
    }
}

/// Draw the grid. A "Qu" square shows as QU.
fn print_board(board: &Board) {
    println!();
    for row in 0..board.size() {
        print!("   ");
        for col in 0..board.size() {
            print!("{:<4}", board.cell(row, col).to_uppercase());
        }
        println!();
    }
    println!();
}

fn print_instructions() {
    println!("Find words in the grid! Each word needs at least {} letters, and each", boggle::MIN_WORD_LENGTH);
    println!("letter must touch the one before it (up, down, left, right or diagonal).");
    println!("You can't use the same square twice in one word.");
    println!("Type your words separated by spaces or commas, or one per line.");
    println!("Words are checked when you finish, and each wrong guess costs {} point.", boggle::WRONG_GUESS_PENALTY);
    println!("Commands:  :done = finish and score    :grid = show the grid again");
    println!("           :help = show this message   :quit = leave the game");
}

/// The inner loop: collect the player's guesses.
/// Returns `Some(list of words)` when they type :done,
/// or `None` if they want to quit the whole game.
fn collect_guesses(board: &Board) -> Option<Vec<String>> {
    print_instructions();
    let mut guesses: Vec<String> = Vec::new();

    loop {
        let line = prompt("> ")?; // input ended -> quit

        if line.is_empty() {
            println!("(Type some words, or :done when you are finished.)");
            continue;
        }

        // A line that starts with a colon is a command.
        if line.starts_with(':') {
            match line.to_lowercase().as_str() {
                ":done" => return Some(guesses),
                ":quit" => return None,
                ":grid" => print_board(board),
                ":help" => print_instructions(),
                _ => println!("I don't know the command '{}'. Type :help to see the commands.", line),
            }
            continue;
        }

        // Otherwise it is one or more guesses.
        for token in boggle::split_line(&line) {
            match boggle::check_format(&token) {
                Ok(word) => {
                    if guesses.contains(&word) {
                        println!("You already entered '{}', so I'll count it only once.", word);
                    } else {
                        guesses.push(word);
                    }
                }
                Err(message) => println!("{}", message),
            }
        }
        println!("({} word(s) so far. Type :done when you are finished.)", guesses.len());
    }
}

/// Show which words scored, which did not, and the final score.
fn print_report(report: &Report) {
    println!();
    println!("========== RESULTS ==========");

    if report.accepted.is_empty() {
        println!("No valid words this time.");
    } else {
        println!("Words that scored:");
        for (word, points) in &report.accepted {
            println!("  {:<15} +{}", word, points);
        }
    }

    if !report.rejected.is_empty() {
        println!("Words that did not count:");
        for (word, reason) in &report.rejected {
            let why = match reason {
                Rejection::NotInDictionary => "not in the dictionary",
                Rejection::NotOnGrid => "can't be made on this grid",
            };
            println!("  {:<15} -{}  ({})", word, boggle::WRONG_GUESS_PENALTY, why);
        }
    }

    println!("Score for this round: {}", report.score);
}
