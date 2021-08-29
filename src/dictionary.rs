use crate::trie::Trie;

use std::iter::FromIterator;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, self};
use std::path::Path;

#[derive(Debug)]
pub struct Dictionary {
    pub words: Vec<String>,
    anagrams: Trie,
}

fn sort_letters(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    chars.sort_by(|a, b| b.cmp(a));
    String::from_iter(chars.iter())
}

impl Dictionary {
    pub fn new(unfiltered_words: Vec<String>) -> Dictionary {
        let words = Dictionary::filter_and_normalize(unfiltered_words);
        let anagrams = Dictionary::new_anagram_trie(&words);
        Dictionary { words, anagrams }
    }

    pub fn new_from_file<P>(path: P) -> io::Result<Dictionary> where P: AsRef<Path> {
        let file = File::open(path)?;
        let words = BufReader::new(file).lines()
            .flat_map(|line| line)
            .collect();
        Ok(Dictionary::new(words))
    }

    pub fn lookup(&self, word: &str) -> Option<&str> {
        for our_word in &self.words {
            if our_word == word {
                return Some(our_word);
            }
        }
        return None;
    }

    pub fn lookup_anagram(&self, word: &str) -> Vec<&str> {
        self.anagrams.lookup(&sort_letters(word)).iter()
            .map(|&idx| self.words[idx].as_ref())
            .collect()
    }

    fn filter_and_normalize(words: Vec<String>) -> Vec<String> {
        words.iter().filter(|word| word.is_ascii() && word.chars().all(char::is_alphabetic))
            .map(|word| word.to_ascii_lowercase())
            .collect()
    }

    fn new_anagram_trie(words: &[String]) -> Trie {
        let mut trie = Trie::new();
        for i in 0..words.len() {
            trie.add(&sort_letters(&words[i]), i);
        }
        trie
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dict_new_from_file() {
        let dict = Dictionary::new_from_file("test_data/dict").unwrap();
        assert!(dict.words.len() > 0);
    }

    #[test]
    fn anagrams() {
        let words = vec!["foo".into(), "bar".into(), "ofo".into()];
        let dict = Dictionary::new(words);
        assert_eq!(dict.lookup_anagram("oof"), vec!["foo", "ofo"]);
        assert_eq!(dict.lookup_anagram("arb"), vec!["bar"]);
        assert_eq!(dict.lookup_anagram("foob").len(), 0);
    }
}
