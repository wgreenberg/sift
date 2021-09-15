use crate::trie::Trie;

use std::iter::FromIterator;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, self};
use std::path::Path;
use bincode::{serialize_into, deserialize_from};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Dictionary {
    pub words: Vec<String>,
    words_trie: Trie,
    anagrams: Trie,
}

pub fn sort_letters(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    chars.sort_by(|a, b| b.cmp(a));
    String::from_iter(chars.iter())
}

impl Dictionary {
    pub fn new(unfiltered_words: Vec<String>) -> Dictionary {
        let words = Dictionary::filter_and_normalize(unfiltered_words);
        let words_trie = Dictionary::new_word_trie(&words);
        let anagrams = Dictionary::new_anagram_trie(&words);
        Dictionary { words, words_trie, anagrams }
    }

    pub fn serialize_to_file<P>(&self, path: P) -> io::Result<()> where P: AsRef<Path> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serialize_into(writer, self).unwrap();
        Ok(())
    }

    pub fn deserialize_from_file<P>(path: P) -> io::Result<Dictionary> where P: AsRef<Path> {
        let file = File::open(path)?;
        Ok(deserialize_from(BufReader::new(file)).unwrap())
    }

    pub fn new_from_file<P>(path: P) -> io::Result<Dictionary> where P: AsRef<Path> {
        let file = File::open(path)?;
        let words = BufReader::new(file).lines()
            .flat_map(|line| line)
            .collect();
        Ok(Dictionary::new(words))
    }

    pub fn lookup(&self, word: &str) -> Vec<&str> {
        self.words_trie.lookup(word).iter()
            .map(|&idx| self.words[idx].as_ref())
            .collect()
    }

    pub fn lookup_anagram(&self, word: &str, sort: bool) -> Vec<&str> {
        let anagrams = if sort {
            self.anagrams.lookup(&sort_letters(word))
        } else {
            self.anagrams.lookup(word)
        };
        anagrams.iter()
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

    fn new_word_trie(words: &[String]) -> Trie {
        let mut trie = Trie::new();
        for i in 0..words.len() {
            trie.add(&words[i], i);

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
        assert_eq!(dict.lookup_anagram("oof", true), vec!["foo", "ofo"]);
        assert_eq!(dict.lookup_anagram("arb", true), vec!["bar"]);
        assert_eq!(dict.lookup_anagram("foob", true).len(), 0);
    }
}
