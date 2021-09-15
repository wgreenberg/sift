use crate::dictionary::{Dictionary, sort_letters};
use std::io;
use std::path::Path;
use regex::Regex;
use itertools::Itertools;

pub struct Sifter {
    dict: Dictionary,
}

impl Sifter {
    pub fn new() -> Sifter {
        Sifter::new_from_dict_path("/etc/dictionaries-common/words").unwrap()
    }

    pub fn load_cached<P>(path: P) -> io::Result<Sifter> where P: AsRef<Path> {
        let dict = Dictionary::deserialize_from_file(path)?;
        Ok(Sifter { dict })
    }

    pub fn save_cache<P>(&self, path: P) -> io::Result<()> where P: AsRef<Path> {
        self.dict.serialize_to_file(path)
    }

    pub fn new_from_dict_path<P>(path: P) -> io::Result<Sifter> where P: AsRef<Path> {
        let dict = Dictionary::new_from_file(path)?;
        Ok(Sifter { dict })
    }

    pub fn anagrams(&self, letters: &str) -> Vec<&str> {
        let mut result = self.dict.lookup_anagram(letters, true);
        result.retain(|&word| word != letters);
        result
    }

    pub fn regex(&self, pattern: &Regex) -> Vec<&str> {
        let whole_word_pattern = format!("^{}$", pattern.as_str());
        let regex = Regex::new(&whole_word_pattern).unwrap();
        self.dict.words.iter()
            .filter(|word| regex.is_match(word))
            .map(std::ops::Deref::deref)
            .collect()
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

    //  - transdelete [n] <letters>: anagram of letters after removing n chars (default 1)
    pub fn transdelete(&self, letters: &str, n: usize) -> Vec<&str> {
        if n == 0 {
            return self.anagrams(letters);
        }
        if n > letters.len() {
            return vec![];
        }
        let mut results = Vec::new();
        for new_word in Sifter::all_deletes(letters, n) {
            results.extend(self.dict.lookup_anagram(&new_word, true));
        }
        return results;
    }

    //  - delete [n] <latters>: words achievable by deleting n letters (default 1)
    pub fn delete(&self, letters: &str, n: usize) -> Vec<&str> {
        let mut results = Vec::new();
        if n == 0 {
            results.extend(self.dict.lookup(letters));
            return results;
        }
        if n > letters.len() {
            return vec![];
        }
        for new_word in Sifter::all_deletes(letters, n) {
            results.extend(self.dict.lookup(&new_word));
        }
        return results;
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

    //  - transadd [n] <letters>: anagram of letters after adding n chars (default 1)
    pub fn transadd(&self, letters: &str, n: usize) -> Vec<&str> {
        if n == 0 {
            return self.anagrams(letters);
        }
        let mut words = Vec::new();
        for wildcard_string in Sifter::all_added_wildcards(&sort_letters(letters), n) {
            words.extend(self.dict.lookup_anagram(&wildcard_string, false));
        }
        words
    }

    //  - add [n] <latters>: words achievable by adding n letters (default 1)
    pub fn add(&self, letters: &str, n: usize) -> Vec<&str> {
        let mut words = Vec::new();
        if n == 0 {
            words.extend(self.dict.lookup(letters));
        } else {
            for wildcard_string in Sifter::all_added_wildcards(letters, n) {
                words.extend(self.dict.lookup(&wildcard_string));
            }
        }
        words
    }

    //  - bank <letters>: words using the same set of letters
    pub fn bank(&self, letters: &str) -> Vec<&str> {
        let mut plus_pattern = String::new();
        for letter in sort_letters(letters).chars().dedup() {
            plus_pattern.push(letter);
            plus_pattern.push('+');
        }
        self.dict.lookup_anagram(&plus_pattern, false)
    }

    //  - change [n] <letters>: achievable by exactly N letter changes (default 1)
    pub fn change(&self, letters: &str, n: usize) -> Vec<&str> {
        let mut words = Vec::new();
        if n == 0 {
            words.extend(self.dict.lookup(letters));
            return words;
        }
        if n > letters.len() {
            return words;
        }
        for combo in Sifter::all_replaced_wildcards(letters, n) {
            words.extend(self.dict.lookup(&combo));
        }
        words.retain(|&word| word != letters);
        words
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::assert_set_equality;

    fn test_sifter() -> Sifter {
        Sifter::new_from_dict_path("test_data/dict").unwrap()
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
    fn test_transdelete() {
        let sifter = test_sifter();
        assert_set_equality(sifter.transdelete("horses", 2), vec![
            "rose",
            "ross",
            "hero",
            "shes",
            "shoe",
            "hess",
            "hose",
        ]);
        assert_set_equality(sifter.transdelete("malls", 0), sifter.anagrams("malls"));
    }

    #[test]
    fn test_delete() {
        let sifter = test_sifter();
        assert_set_equality(sifter.delete("horses", 2), vec!["hose"]);
        assert_set_equality(sifter.delete("small", 0), vec!["small"]);
        assert_set_equality(sifter.delete("smpll", 0), vec![]);
    }

    #[test]
    fn test_all_added_wildcards() {
        assert_set_equality(Sifter::all_added_wildcards("aa", 3), vec![
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
    fn test_transadd() {
        let sifter = test_sifter();
        assert_set_equality(sifter.transadd("small", 0), vec!["malls"]);
        assert_set_equality(sifter.transadd("horse", 1), vec![
            "others",
            "heroes",
            "horses",
            "rhodes",
            "shores",
        ]);
        assert_set_equality(sifter.transadd("horse", 2), vec![
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