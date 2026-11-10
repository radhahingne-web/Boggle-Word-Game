//! A trie (say "try"): a tree where each step down the tree is one letter.
//!
//! Example: after inserting "cat" and "car" the trie looks like this:
//!
//! ```text
//!   (start) -- c -- a -- t   <- "cat" ends here
//!                      \- r  <- "car" ends here
//! ```
//!
//! To check a word we just follow its letters downwards.
//! If we run out of tree, the word is not there.
//! This is much faster than searching through a list of 370,000 words.

use std::collections::HashMap;

/// One spot in the tree.
struct TrieNode {
    /// The letters that can come next, each leading to another spot.
    children: HashMap<char, TrieNode>,
    /// True if a complete word ends exactly here.
    is_word: bool,
}

impl TrieNode {
    fn new() -> TrieNode {
        TrieNode {
            children: HashMap::new(),
            is_word: false,
        }
    }
}

/// The whole trie. (The `TrieNode`s are private - outside code can only
/// use the three functions below. That is "data encapsulation".)
pub struct Trie {
    root: TrieNode,
}

impl Trie {
    /// Make an empty trie.
    pub fn new() -> Trie {
        Trie {
            root: TrieNode::new(),
        }
    }

    /// Make a trie that holds all the given words.
    /// We only *borrow* the list (`&`), so the caller keeps their list.
    pub fn from_words(words: &[String]) -> Trie {
        let mut trie = Trie::new();
        for word in words {
            trie.insert(word);
        }
        trie
    }

    /// Add one word to the trie.
    pub fn insert(&mut self, word: &str) {
        let mut node = &mut self.root;
        for letter in word.chars() {
            // Go to the child for this letter, creating it if it is missing.
            node = node.children.entry(letter).or_insert_with(TrieNode::new);
        }
        node.is_word = true; // the last letter marks the end of a word
    }

    /// Is this exact word in the trie?
    pub fn contains(&self, word: &str) -> bool {
        let mut node = &self.root;
        for letter in word.chars() {
            match node.children.get(&letter) {
                Some(next) => node = next,
                None => return false, // ran out of tree
            }
        }
        node.is_word // "ca" is only a start of "cat", not a word itself
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_trie() -> Trie {
        let mut trie = Trie::new();
        trie.insert("cat");
        trie.insert("car");
        trie.insert("cart");
        trie
    }

    #[test]
    fn finds_words_that_were_inserted() {
        let trie = small_trie();
        assert!(trie.contains("cat"));
        assert!(trie.contains("car"));
        assert!(trie.contains("cart"));
    }

    #[test]
    fn does_not_find_other_words() {
        let trie = small_trie();
        assert!(!trie.contains("dog"));
        assert!(!trie.contains("cats"));
    }

    #[test]
    fn a_prefix_is_not_a_word() {
        let trie = small_trie();
        assert!(!trie.contains("ca"));
        assert!(!trie.contains("c"));
        assert!(!trie.contains(""));
    }

    #[test]
    fn can_build_from_a_list() {
        let words = vec!["one".to_string(), "two".to_string()];
        let trie = Trie::from_words(&words);
        assert!(trie.contains("one"));
        assert!(trie.contains("two"));
        assert!(!trie.contains("three"));
    }
}
