use crate::sifter::Sifter;
use regex::Regex;

#[derive(Clone)]
pub enum SiftCommand {
    RegExp(Regex),
    Anagram(String),
    Bank(String),
    Transdelete(String, usize),
    Transadd(String, usize),
    Delete(String, usize),
    Add(String, usize),
    Change(String, usize),
}
use SiftCommand::*;

impl SiftCommand {
    pub fn run<'a>(&self, sifter: &'a Sifter) -> Vec<&'a str> {
        match self {
            RegExp(regex) => sifter.regex(&regex),
            Anagram(letters) => sifter.anagrams(&letters),
            Bank(letters) => sifter.bank(&letters),
            Transdelete(letters, n) => sifter.transdelete(&letters, *n),
            Transadd(letters, n) => sifter.transadd(&letters, *n),
            Delete(letters, n) => sifter.delete(&letters, *n),
            Add(letters, n) => sifter.add(&letters, *n),
            Change(letters, n) => sifter.change(&letters, *n),
        }
    }

    pub fn substitute(&self, word: &str) -> SiftCommand {
        match self {
            RegExp(regex) => {
                let new_regex = Regex::new(&regex.as_str().replace("%", word)).unwrap();
                RegExp(new_regex)
            },
            Anagram(letters) => Anagram(letters.replace("%", word)),
            Bank(letters) => Bank(letters.replace("%", word)),
            Transdelete(letters, n) => Transdelete(letters.replace("%", word), *n),
            Transadd(letters, n) => Transadd(letters.replace("%", word), *n),
            Delete(letters, n) => Delete(letters.replace("%", word), *n),
            Add(letters, n) => Add(letters.replace("%", word), *n),
            Change(letters, n) => Change(letters.replace("%", word), *n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute() {
        let cmd = SiftCommand::RegExp(Regex::new("..%..%").unwrap());
        if let SiftCommand::RegExp(r) = cmd.substitute("foobar") {
            assert_eq!(r.as_str(), "..foobar..foobar");
        } else {
            panic!("got wrong variant back from substitute");
        }

        let cmd = SiftCommand::Anagram("blah%blah%blah".to_string());
        if let SiftCommand::Anagram(r) = cmd.substitute("x") {
            assert_eq!(r, "blahxblahxblah");
        } else {
            panic!("got wrong variant back from substitute");
        }

        let cmd = SiftCommand::Transdelete("blah%blah%blah".to_string(), 5);
        if let SiftCommand::Transdelete(r, 5) = cmd.substitute("x") {
            assert_eq!(r, "blahxblahxblah");
        } else {
            panic!("got wrong variant back from substitute");
        }
    }
}
