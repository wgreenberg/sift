use crate::trie::Trie;
use crate::argparse::SiftError;

use std::iter::FromIterator;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
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

    pub fn write_cache<W>(&self, writer: W) -> Result<(), SiftError> where W: Write {
        serialize_into(BufWriter::new(writer), self)
            .map_err(|err| {
                eprintln!("{}", err);
                SiftError::SerializationError
            })
    }

    pub fn new_from_cache<R>(data: R) -> Result<Dictionary, SiftError> where R: Read {
        deserialize_from(BufReader::new(data))
            .map_err(|err| {
                eprintln!("{}", err);
                SiftError::DeserializationError
            })
    }

    pub fn new_from_words<R>(data: R) -> Dictionary where R: Read {
        let words = BufReader::new(data).lines()
            .flat_map(|line| line)
            .collect();
        Dictionary::new(words)
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
    fn anagrams() {
        let words = vec!["foo".into(), "bar".into(), "ofo".into()];
        let dict = Dictionary::new(words);
        assert_eq!(dict.lookup_anagram("oof", true), vec!["foo", "ofo"]);
        assert_eq!(dict.lookup_anagram("arb", true), vec!["bar"]);
        assert_eq!(dict.lookup_anagram("foob", true).len(), 0);
    }
}
