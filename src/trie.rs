use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Trie {
    root: TrieNode,
}

impl Trie {
    pub fn new() -> Trie {
        Trie { root: TrieNode::new_root() }
    }

    pub fn lookup(&self, path: &str) -> Vec<usize> {
        self.root.lookup(path)
    }

    pub fn add(&mut self, path: &str, idx: usize) {
        self.root.add(path, idx);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TrieNode {
    letter: Option<char>,
    words: Vec<usize>,
    nodes: Vec<TrieNode>,
}

impl TrieNode {
    fn new_root() -> TrieNode {
        TrieNode::new(None)
    }

    fn new(letter: Option<char>) -> TrieNode {
        TrieNode { letter, words: Vec::new(), nodes: Vec::new() }
    }

    fn add(&mut self, path: &str, idx: usize) {
        let mut path_letters = path.chars();
        match path_letters.next() {
            None => self.words.push(idx),
            Some(letter) => {
                for node in &mut self.nodes {
                    if node.letter == Some(letter) {
                        node.add(path_letters.as_str(), idx);
                        return;
                    }
                }
                let mut new_node = TrieNode::new(Some(letter));
                new_node.add(path_letters.as_str(), idx);
                self.nodes.push(new_node);
            },
        }
    }

    fn lookup(&self, path: &str) -> Vec<usize> {
        let mut path_letters = path.chars();
        match path_letters.next() {
            None => self.words.clone(),
            Some('+') => {
                let mut matches = Vec::new();
                let mut reassembled = String::from("+");
                reassembled.push_str(path_letters.as_str());
                for node in &self.nodes {
                    if node.letter == self.letter {
                        matches.extend(node.lookup(&reassembled));
                    }
                }
                matches.extend(self.lookup(path_letters.as_str()));
                matches
            },
            Some('.') => {
                let mut matches = Vec::new();
                for node in &self.nodes {
                    matches.extend(node.lookup(path_letters.as_str()));
                }
                matches
            },
            Some(letter) => {
                for node in &self.nodes {
                    if node.letter == Some(letter) {
                        return node.lookup(path_letters.as_str());
                    }
                }
                return vec![];
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::assert_set_equality;

    #[test]
    fn test_add() {
        let mut trie = Trie::new();
        trie.add("foo", 1);
        assert_eq!(trie.root.nodes[0].letter, Some('f'));
        assert_eq!(trie.root.nodes[0].nodes[0].letter, Some('o'));
        assert_eq!(trie.root.nodes[0].nodes[0].nodes[0].letter, Some('o'));
        assert_eq!(trie.root.nodes[0].nodes[0].nodes[0].words, vec![1]);
        trie.add("f", 2);
        assert_eq!(trie.root.nodes[0].words, vec![2]);
    }

    #[test]
    fn test_lookup() {
        let mut trie = Trie::new();
        trie.add("foo", 1);
        assert_eq!(trie.lookup("f").len(), 0);
        assert_eq!(trie.lookup("fo").len(), 0);
        assert_eq!(trie.lookup("foo"), vec![1]);
        trie.add("f", 2);
        assert_eq!(trie.lookup("f"), vec![2]);
        assert_eq!(trie.lookup("fo").len(), 0);
        assert_eq!(trie.lookup("foo"), vec![1]);
    }

    #[test]
    fn test_repeats() {
        let mut trie = Trie::new();
        trie.add("ab", 1);
        trie.add("aab", 2);
        trie.add("aaab", 3);
        trie.add("aaa", 0);
        trie.add("aaaab", 4);
        trie.add("aaaabb", 0);
        assert_set_equality(trie.lookup("a+b"), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_wildcards() {
        let mut trie = Trie::new();
        trie.add("aaaa", 1);
        trie.add("aaba", 2);
        trie.add("aaca", 3);
        trie.add("abaa", 4);
        assert_eq!(trie.lookup("aa.a"), vec![1, 2, 3]);
    }
}
