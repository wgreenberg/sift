#[derive(Debug)]
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

#[derive(Debug)]
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
}
