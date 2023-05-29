use std::collections::HashMap;

fn main() {
    let mut trie = Trie::new();
    trie.insert("apple".to_string());
    println!("{}", trie.search("apple".to_string()));   // returns true
    println!("{}", trie.search("app".to_string()));     // returns false
    println!("{}", trie.starts_with("app".to_string())); // returns true
    trie.insert("app".to_string());   
    println!("{}", trie.search("app".to_string()));     // returns true
}





struct TrieNode {
    children: HashMap<char,TrieNode>,
    is_word: bool,
}

impl TrieNode {
    fn new() -> Self {
        TrieNode {children: HashMap::new(), is_word: false}
    }   
}


struct Trie {
    root: TrieNode,
}

impl Trie {

    fn new() -> Self {
        Trie {root: TrieNode::new()}
    }
    
    fn insert(&mut self, word: String) {
        let mut current_node = &mut self.root;
        
        for c in word.chars() {
            let next_node = current_node.children.entry(c)
                            .or_insert(TrieNode::new());
            current_node = next_node;
        }
        current_node.is_word = true;
    }
    
    fn search(&self, word: String) -> bool {
        let mut current_node = &self.root;
        
        for c in word.chars() {
            match current_node.children.get(&c) {
                Some(next_node) => current_node = next_node,
                None => return false,
            }
        }
        
        return current_node.is_word;
    }
    
    fn starts_with(&self, prefix: String) -> bool {
        let mut current_node = &self.root;
        
        for c in prefix.chars() {
            match current_node.children.get(&c) {
                Some(next_node) => current_node = next_node,
                None => return false,
            }
        }
        
        return true;
    }
}