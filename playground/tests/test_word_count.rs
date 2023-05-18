use playground::WordCounter;

#[cfg(test)]
// Our first test
#[test]
fn first_test() {
    assert!(true);
    assert_ne!(false, true);
    assert_eq!(1 + 1, 2);
}

// This is a test for the word counter
#[test]
fn test_word_count() {
    let mut word_counter = WordCounter::new();

    let random_words = [
        "this", "is", "just", "a", "list", "of", "random", "words.", "a", "word", "please",
    ];

    // this adds the words to the hashmap and increments the number of times the word appears
    for word in random_words.iter() {
        word_counter.increment(word);
    }

    assert!(random_words.len() > 0);

    // we should expect 10 because the word "a" is repeated twice, that means "a" will map to 2
    assert!(word_counter.0.len() == 10_usize);
}