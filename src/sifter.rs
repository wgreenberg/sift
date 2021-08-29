use crate::dictionary::Dictionary;
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

    pub fn new_from_dict_path<P>(path: P) -> io::Result<Sifter> where P: AsRef<Path> {
        let dict = Dictionary::new_from_file(path)?;
        Ok(Sifter { dict })
    }

    pub fn anagrams(&self, letters: &str) -> Vec<&str> {
        let mut result = self.dict.lookup_anagram(letters);
        result.retain(|&word| word != letters);
        result
    }

    pub fn regex(&self, pattern: &str) -> Result<Vec<&str>, regex::Error> {
        let whole_word_pattern = format!("^{}$", pattern);
        let regex = Regex::new(&whole_word_pattern)?;
        let words = self.dict.words.iter()
            .filter(|word| regex.is_match(word))
            .map(std::ops::Deref::deref)
            .collect();
        Ok(words)
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
            results.extend(self.dict.lookup_anagram(&new_word));
        }
        return results;
    }

    //  - delete [n] <latters>: words achievable by deleting n letters (default 1)
    pub fn delete(&self, letters: &str, n: usize) -> Vec<&str> {
        if n == 0 {
            return match self.dict.lookup(letters) {
                Some(word) => vec![word],
                None => vec![],
            };
        }
        if n > letters.len() {
            return vec![];
        }
        let mut results = Vec::new();
        for new_word in Sifter::all_deletes(letters, n) {
            if let Some(word) = self.dict.lookup(&new_word) {
                results.push(word);
            }
        }
        return results;
    }

    //  - transadd [n] <letters>: anagram of letters after adding n chars (default 1)
    //  - bank <letters>: words using the same set of letters
    //  - substring <letters>: words contained in the substring
    //  - add [n] <latters>: words achievable by adding n letters (default 1)
    //  - change [n] <letters>: achievable by exactly N letter changes (default 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::hash::Hash;
    use std::fmt::Debug;

    pub fn assert_set_equality<T>(got: Vec<T>, expected: Vec<T>)
        where T: Clone + Eq + Hash + Debug {
        let got_hash: HashSet<T> = got.iter().cloned().collect();
        let expected_hash: HashSet<T> = expected.iter().cloned().collect();
        if got_hash != expected_hash {
            let unwanted: HashSet<&T> = got_hash.difference(&expected_hash).collect();
            let needed: HashSet<&T> = expected_hash.difference(&got_hash).collect();
            panic!("set inequality! expected len {}, got {}\nmissing {:?}\nunwanted {:?}",
                expected_hash.len(), got_hash.len(), needed, unwanted);
        }
    }

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
        assert_set_equality(sifter.regex("sm..l").unwrap(), vec!["small"]);
        assert_set_equality(sifter.regex(".{5}").unwrap(), vec!["malls", "small"]);
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
}
