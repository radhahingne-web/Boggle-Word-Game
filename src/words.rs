//! Loading the dictionary (the list of allowed words).
//!
//! The dictionary is a plain text file with one word on each line.

use std::fs;

/// Is this word made only of the lowercase letters a-z?
/// This quietly removes capitalised words ("Paris") and hyphenated words ("well-known").
pub fn is_plain_word(word: &str) -> bool {
    !word.is_empty() && word.chars().all(|c| c.is_ascii_lowercase())
}

/// Turn the text of a dictionary file into a sorted list of words.
/// Bad lines are skipped and duplicates are removed.
pub fn words_from_text(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    for line in text.lines() {
        let word = line.trim();
        if is_plain_word(word) {
            words.push(word.to_string());
        }
    }
    words.sort();
    words.dedup(); // remove words that appear twice in a row (the list is sorted)
    words
}

/// Read the dictionary file at `path`.
/// `Result` means "either `Ok(answer)` or `Err(what went wrong)`".
pub fn load_words(path: &str) -> Result<Vec<String>, String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(words_from_text(&text)),
        Err(error) => Err(format!(
            "Could not read the word list '{}': {}\n\
             Make sure the file is in the folder you ran the game from.",
            path, error
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_words_are_accepted() {
        assert!(is_plain_word("hello"));
    }

    #[test]
    fn capitals_hyphens_and_empty_are_rejected() {
        assert!(!is_plain_word("Paris"));
        assert!(!is_plain_word("well-known"));
        assert!(!is_plain_word("don't"));
        assert!(!is_plain_word(""));
    }

    #[test]
    fn text_is_cleaned_sorted_and_deduplicated() {
        let words = words_from_text("pear\r\nApple\nfig\npear\nwell-known\n\nbanana\n");
        assert_eq!(words, vec!["banana", "fig", "pear"]);
    }

    #[test]
    fn missing_file_gives_an_error() {
        assert!(load_words("this_file_does_not_exist.txt").is_err());
    }
}
