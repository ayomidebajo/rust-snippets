use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::BufRead;
use std::io::BufReader;

#[derive(Debug)]
pub struct WordCounter(pub HashMap<String, u64>);

impl WordCounter {
    pub fn new() -> WordCounter {
        WordCounter(HashMap::new())
    }
    pub fn increment(&mut self, word: &str) {
        let key = word.to_string();
        let count = self.0.entry(key).or_insert(0);
        *count += 1;
    }
    pub fn display(&self, filter: u64) {
        // keep data in a vector for sorting by storing it the same structure as the hashmap
        let mut vec_count: Vec<(&String, &u64)> = self.0.iter().collect();
        // sort by value
        vec_count.sort_by(|a, b| a.1.cmp(b.1));
        // print the sorted vector
        for (key, value) in vec_count {
            // print only the words that have a count greater than the filter
            if value > &filter {
                println!("{}: {}", key, value);
            }
        }
    }
}

// mod food {
//     pub struct Cake;
//     struct Smoothie;
//     struct Fruit;
// }
// create a directory for this and move it to its seperate directory
#[allow(dead_code)]
fn main() {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() < 2 {
        panic!("Please provide a filename!");
    }
    let filename = &arguments[1];

    let file = File::open(filename).expect("Could not open file");

    let reader = BufReader::new(file);

    let mut word_counter = WordCounter::new();

    for line in reader.lines() {
        let line = line.expect("Could not read line");
        let words = line.split(" ");

        for word in words {
            if word == "" {
                continue;
            } else {
                word_counter.increment(&word);
            }
        }
    }
    // let eat = food::Cake;
    word_counter.display(1);
}


