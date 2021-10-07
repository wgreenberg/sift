use crate::dictionary::{Dictionary, sort_letters};
use crate::argparse::SiftError;
use std::path::Path;
use std::collections::HashSet;
use std::io::prelude::*;
use regex::Regex;
use itertools::Itertools;
use wasm_bindgen::prelude::*;
use std::fs::File;

#[wasm_bindgen]
pub struct Sifter {
    dict: Dictionary,
}

fn create<P>(path: P) -> Result<File, SiftError> where P: AsRef<Path> {
    File::create(path).map_err(|err| {
        eprintln!("{}", err);
        SiftError::FileIOError
    })
}

fn open<P>(path: P) -> Result<File, SiftError> where P: AsRef<Path> {
    File::open(path).map_err(|err| {
        eprintln!("{}", err);
        SiftError::FileIOError
    })
}

fn all_added_wildcards(letters: &str, n: usize) -> Vec<String> {
    let mut words = Vec::new();
    let orig_chars: Vec<char> = letters.chars().collect();
    for combo in (0..letters.len() + n).combinations(letters.len()) {
        let mut new_word = vec!['.'; letters.len() + n];
        for (letters_i, &new_word_i) in combo.iter().enumerate() {
            new_word[new_word_i] = orig_chars[letters_i];
        }
        words.push(new_word.iter().collect());
    }
    words
}

fn all_deletes(letters: &str, n: usize) -> Vec<String> {
    let mut words = Vec::new();
    for combo in (0..letters.len()).combinations(n) {
        let new_word: String = letters.chars().enumerate().filter_map(|(i, c)| {
            if combo.contains(&i) {
                None
            } else {
                Some(c)
            }
        }).collect();
        words.push(new_word);
    }
    words
}

fn all_replaced_wildcards(letters: &str, n: usize) -> Vec<String> {
    let mut words = Vec::new();
    for combo in (0..letters.len()).combinations(n) {
        let new_word: String = letters.chars().enumerate().map(|(i, c)| {
            if combo.contains(&i) {
                '.'
            } else {
                c
            }
        }).collect();
        words.push(new_word);
    }
    words
}

impl Sifter {
    pub fn new_from_cache<R>(data: R) -> Result<Sifter, SiftError> where R: Read {
        Ok(Sifter { dict: Dictionary::new_from_cache(data)? })
    }

    pub fn new_from_cache_file<P>(path: P) -> Result<Sifter, SiftError> where P: AsRef<Path> {
        Sifter::new_from_cache(open(path)?)
    }

    pub fn save_cache_file<P>(&self, path: P) -> Result<(), SiftError> where P: AsRef<Path> {
        self.dict.write_cache(create(path)?)
    }

    pub fn new_from_words<R>(data: R) -> Sifter where R: Read {
        Sifter { dict: Dictionary::new_from_words(data) }
    }

    pub fn new_from_words_file<P>(path: P) -> Result<Sifter, SiftError> where P: AsRef<Path> {
        Ok(Sifter::new_from_words(open(path)?))
    }

    pub fn anagrams(&self, letters: &str) -> Vec<&str> {
        self.dict.lookup_anagram(letters, true)
            .into_iter()
            .filter(|&word| word != letters)
            .collect()
    }

    pub fn regex(&self, pattern: &Regex) -> Vec<&str> {
        let whole_word_pattern = format!("^{}$", pattern.as_str());
        let regex = Regex::new(&whole_word_pattern).unwrap();
        self.dict.words()
            .into_iter()
            .filter(|word| regex.is_match(word))
            .collect()
    }

    pub fn transpose_delete(&self, letters: &str, n: usize) -> Vec<&str> {
        if n > letters.len() {
            return vec![];
        }
        let mut results = HashSet::new();
        for new_word in all_deletes(letters, n) {
            results.extend(self.dict.lookup_anagram(&new_word, true));
        }
        results.remove(letters);
        results.into_iter().collect()
    }

    pub fn delete(&self, letters: &str, n: usize) -> Vec<&str> {
        if n > letters.len() {
            return vec![];
        }
        let mut results = HashSet::new();
        for new_word in all_deletes(letters, n) {
            results.extend(self.dict.lookup(&new_word));
        }
        results.into_iter().collect()
    }


    pub fn transpose_add(&self, letters: &str, n: usize) -> Vec<&str> {
        let mut results = HashSet::new();
        for wildcard_string in all_added_wildcards(&sort_letters(letters), n) {
            results.extend(self.dict.lookup_anagram(&wildcard_string, false));
        }
        results.remove(letters);
        results.into_iter().collect()
    }

    pub fn add(&self, letters: &str, n: usize) -> Vec<&str> {
        let mut results = HashSet::new();
        for wildcard_string in all_added_wildcards(letters, n) {
            results.extend(self.dict.lookup(&wildcard_string));
        }
        results.into_iter().collect()
    }

    pub fn bank(&self, letters: &str) -> Vec<&str> {
        let mut plus_pattern = String::new();
        for letter in sort_letters(letters).chars().dedup() {
            plus_pattern.push(letter);
            plus_pattern.push('+');
        }
        self.dict.lookup_anagram(&plus_pattern, false)
            .into_iter()
            .collect()
    }

    pub fn change(&self, letters: &str, n: usize) -> Vec<&str> {
        if n > letters.len() {
            return vec![];
        }
        let mut results = HashSet::new();
        for combo in all_replaced_wildcards(letters, n) {
            results.extend(self.dict.lookup(&combo));
        }
        results.remove(letters);
        results.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::assert_set_equality;

    fn test_sifter() -> Sifter {
        Sifter::new_from_words_file("test_data/dict").unwrap()
    }

    #[test]
    fn test_anagrams() {
        let sifter = test_sifter();
        assert_set_equality(sifter.anagrams("malls"), vec!["small"]);
        assert_set_equality(sifter.anagrams("small"), vec!["malls"]);
    }

    #[test]
    fn test_regex() {
        let sifter = test_sifter();
        assert_set_equality(sifter.regex(&Regex::new("sm..l").unwrap()), vec!["small"]);
        assert_set_equality(sifter.regex(&Regex::new(".{5}").unwrap()), vec![
            "malls",
            "small",
            "eater",
            "treat",
            "terra",
        ]);
    }

    #[test]
    fn test_transpose_delete() {
        let sifter = test_sifter();
        assert_set_equality(sifter.transpose_delete("horses", 2), vec![
            "rose",
            "ross",
            "hero",
            "shes",
            "shoe",
            "hess",
            "hose",
        ]);
        assert_set_equality(sifter.transpose_delete("malls", 0), sifter.anagrams("malls"));
    }

    #[test]
    fn test_delete() {
        let sifter = test_sifter();
        assert_set_equality(sifter.delete("horses", 2), vec!["hose"]);
        assert_set_equality(sifter.delete("horsess", 1), vec!["horses"]);
        assert_set_equality(sifter.delete("small", 0), vec!["small"]);
        assert_set_equality(sifter.delete("smpll", 0), vec![]);
    }

    #[test]
    fn test_all_replaced_wildcards() {
        assert_set_equality(all_replaced_wildcards("aa", 0), vec![
            "aa".to_string(),
        ]);
        assert_set_equality(all_replaced_wildcards("aa", 1), vec![
            ".a".to_string(),
            "a.".to_string(),
        ]);
        assert_set_equality(all_replaced_wildcards("aaa", 2), vec![
            "..a".to_string(),
            "a..".to_string(),
            ".a.".to_string(),
        ]);
    }

    #[test]
    fn test_all_deletes() {
        assert_set_equality(all_deletes("abc", 0), vec![
            "abc".to_string(),
        ]);
        assert_set_equality(all_deletes("abc", 1), vec![
            "bc".to_string(),
            "ab".to_string(),
            "ac".to_string(),
        ]);
    }

    #[test]
    fn test_all_added_wildcards() {
        assert_set_equality(all_added_wildcards("aa", 0), vec![
            "aa".to_string(),
        ]);
        assert_set_equality(all_added_wildcards("aa", 3), vec![
            "...aa".to_string(),
            "..a.a".to_string(),
            "..aa.".to_string(),
            ".a.a.".to_string(),
            ".aa..".to_string(),
            "a.a..".to_string(),
            "aa...".to_string(),
            ".a..a".to_string(),
            "a...a".to_string(),
            "a..a.".to_string(),
        ]);
    }

    #[test]
    fn test_transpose_add() {
        let sifter = test_sifter();
        assert_set_equality(sifter.transpose_add("small", 0), vec!["malls"]);
        assert_set_equality(sifter.transpose_add("horse", 1), vec![
            "others",
            "heroes",
            "horses",
            "rhodes",
            "shores",
        ]);
        assert_set_equality(sifter.transpose_add("horse", 2), vec![
            "mothers",
            "shorter",
            "porsche",
            "holders",
            "horsley",
            "thorsen",
            "horsely",
        ]);
    }

    #[test]
    fn test_add() {
        let sifter = test_sifter();
        assert_set_equality(sifter.add("small", 0), vec!["small"]);
        assert_set_equality(sifter.add("horse", 2), vec![
            "horsley",
            "thorsen",
            "horsely",
        ]);
    }

    #[test]
    fn test_bank() {
        let sifter = test_sifter();
        assert_set_equality(sifter.bank("rate"), vec![
            "retreat",
            "treat",
            "terra",
            "tear",
            "eater",
        ]);
    }

    #[test]
    fn test_change() {
        let sifter = test_sifter();
        assert_set_equality(sifter.change("horses", 2), vec![
            "forces",
            "heroes",
            "losses",
            "holmes",
            "housed",
            "forbes",
        ]);
    }
}
