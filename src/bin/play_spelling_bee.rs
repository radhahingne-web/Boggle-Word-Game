//! Play Spelling Bee in the terminal.
//!
//! This file only deals with talking to the player (printing and typing).
//! All the rules live in `src/spelling_bee.rs`.
//!
//! Run it with:  cargo run --release --bin play_spelling_bee

use word_games::console::prompt;
use word_games::rng::Rng;
use word_games::spelling_bee::{self, GuessResult, Puzzle, MIN_WORD_LENGTH};
use word_games::words;

/// The dictionary file: one word per line.
const DICTIONARY_FILE: &str = "words.txt";

fn main() {
    println!("=====================================");
    println!("        Welcome to SPELLING BEE");
    println!("=====================================");
    println!("Loading the word list...");

    // Load the dictionary. If that fails, say why and stop.
    let dictionary = match words::load_words(DICTIONARY_FILE) {
        Ok(list) => list,
        Err(message) => {
            println!("{}", message);
            return;
        }
    };

    // Make the 7 letters for today's puzzle.
    let mut rng = Rng::from_clock();
    let letters = match spelling_bee::generate_letters(&dictionary, &mut rng) {
        Some(letters) => letters,
        None => {
            println!("The word list has no word with 7 different letters, so I cannot make a puzzle.");
            return;
        }
    };

    // `letters` is moved into the puzzle. We cannot use it again here.
    let mut puzzle = Puzzle::new(letters, &dictionary);

    show_help(&puzzle);
    play(&mut puzzle);
    show_final_results(&puzzle);
}

/// The main loop: keep asking for guesses until the player quits or wins.
/// We pass the puzzle as `&mut` (a mutable borrow) because guessing changes it.
fn play(puzzle: &mut Puzzle) {
    loop {
        show_letters(puzzle);
        show_found_words(puzzle);

        let input = match prompt("Your guess (or :help): ") {
            Some(text) => text,
            None => return, // input ended: treat it like quitting
        };

        if input.is_empty() {
            println!("Please type a word and press Enter.");
            continue;
        }

        // Commands start with a colon, so they can never be mistaken for words.
        if input.starts_with(':') {
            match input.to_lowercase().as_str() {
                ":quit" => return,
                ":help" => show_help(puzzle),
                ":hint" => show_hint(puzzle),
                _ => println!("I don't know the command '{}'. Type :help to see the commands.", input),
            }
            continue;
        }

        if input.contains(' ') {
            println!("Please type just one word at a time.");
            continue;
        }

        // Check the guess and tell the player what happened.
        match puzzle.guess(&input) {
            GuessResult::Correct { points, is_pangram } => {
                if is_pangram {
                    println!("PANGRAM! Amazing! '{}' uses all 7 letters: +{} points!", input.to_lowercase(), points);
                } else {
                    println!("Nice! '{}' scores {} point(s).", input.to_lowercase(), points);
                }
            }
            GuessResult::TooShort => {
                println!("Too short! Words need at least {} letters.", MIN_WORD_LENGTH);
            }
            GuessResult::BadLetters(bad) => {
                let list: Vec<String> = bad.iter().map(|c| c.to_ascii_uppercase().to_string()).collect();
                println!("Oops, today's letters don't include: {}.", list.join(", "));
            }
            GuessResult::MissingMainLetter => {
                println!("Every word must contain the main letter, {}.", puzzle.main_letter().to_ascii_uppercase());
            }
            GuessResult::AlreadyGuessed => {
                println!("You already found '{}'. Try a different word!", input.to_lowercase());
            }
            GuessResult::NotInList => {
                println!("Hmm, '{}' isn't in my word list. Try another!", input.to_lowercase());
            }
        }

        if puzzle.is_complete() {
            show_found_words(puzzle);
            println!("*** QUEEN BEE! You found every single word! ***");
            return;
        }
    }
}

/// Show the 7 letters as a honeycomb of hexagons.
/// The main letter sits in the middle cell; the other six surround it.
fn show_letters(puzzle: &Puzzle) {
    // `letters()` has the main letter first, so index 0 is the centre.
    let l: Vec<char> = puzzle.letters().iter().map(|c| c.to_ascii_uppercase()).collect();

    println!();
    println!("      ___");
    println!("  ___/ {} \\___", l[1]);
    println!(" / {} \\___/ {} \\", l[2], l[3]);
    println!(" \\___/ [{}] \\___/", l[0]);
    println!(" / {} \\___/ {} \\", l[4], l[5]);
    println!(" \\___/ {} \\___/", l[6]);
    println!("     \\___/");
    println!();
    println!("Main letter: [{}]   Score: {} points", l[0], puzzle.score());
}

/// Show the words found so far, in alphabetical order, in three columns.
fn show_found_words(puzzle: &Puzzle) {
    let words = puzzle.guessed();
    if words.is_empty() {
        println!("(no words found yet)");
        return;
    }
    println!("Words found ({}):", words.len());
    print_in_columns(words);
}

/// Print words three to a row, each padded to the same width.
fn print_in_columns(words: &[String]) {
    for (index, word) in words.iter().enumerate() {
        print!("  {:<16}", word);
        if index % 3 == 2 {
            println!(); // finished a row of three
        }
    }
    if words.len() % 3 != 0 {
        println!(); // finish the last, shorter row
    }
}

/// Explain the rules and the commands.
fn show_help(puzzle: &Puzzle) {
    println!();
    println!("HOW TO PLAY");
    println!("  * Make words using only the 7 letters (you may reuse letters).");
    println!("  * Every word needs at least {} letters and must use the main letter, {}.", MIN_WORD_LENGTH, puzzle.main_letter().to_ascii_uppercase());
    println!("  * 4-letter words score 1 point; longer words score 1 point per letter.");
    println!("  * A pangram (uses all 7 letters) earns {} bonus points!", spelling_bee::PANGRAM_BONUS);
    println!("COMMANDS");
    println!("  :hint   show how many words there are of each length / start");
    println!("  :help   show this message again");
    println!("  :quit   give up and see the words you missed");
}

/// Show the hint grid and the two-letter list.
fn show_hint(puzzle: &Puzzle) {
    println!();
    println!(
        "WORDS: {}, POINTS: {}, PANGRAMS: {}",
        puzzle.answer_count(),
        puzzle.total_points(),
        puzzle.pangram_count()
    );

    // --- The grid: rows = first letter, columns = word length ---
    let shortest = MIN_WORD_LENGTH;
    let longest = puzzle.longest_answer();

    print!("     ");
    for length in shortest..=longest {
        print!("{:>4}", length);
    }
    println!("   Total");

    let mut column_totals = vec![0; longest - shortest + 1];
    let mut grand_total = 0;
    for (letter, counts) in puzzle.hint_grid() {
        print!("  {}: ", letter.to_ascii_uppercase());
        let mut row_total = 0;
        for (index, count) in counts.iter().enumerate() {
            if *count == 0 {
                print!("{:>4}", "-");
            } else {
                print!("{:>4}", count);
            }
            column_totals[index] += *count;
            row_total += *count;
        }
        println!("{:>8}", row_total);
        grand_total += row_total;
    }
    print!("  Σ:  ");
    for total in &column_totals {
        print!("{:>4}", total);
    }
    println!("{:>8}", grand_total);

    // --- The two-letter list: AC-3 AO-1 ... ---
    println!();
    println!("Two-letter list:");
    let mut previous_first_letter = ' ';
    for (start, count) in puzzle.two_letter_counts() {
        let first_letter = start.chars().next().unwrap();
        if first_letter != previous_first_letter && previous_first_letter != ' ' {
            println!(); // new first letter = new row
        }
        print!("{}-{}  ", start.to_uppercase(), count);
        previous_first_letter = first_letter;
    }
    println!();
}

/// Show the final score and the words the player did not find.
fn show_final_results(puzzle: &Puzzle) {
    println!();
    println!("=====================================");
    println!("Thanks for playing!");
    println!(
        "You found {} of {} words and scored {} of {} points.",
        puzzle.guessed().len(),
        puzzle.answer_count(),
        puzzle.score(),
        puzzle.total_points()
    );

    let missed = puzzle.missed_words();
    if !missed.is_empty() {
        println!("Words you did not find ({}):", missed.len());
        print_in_columns(&missed);
    }
}
