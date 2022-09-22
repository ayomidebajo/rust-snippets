use std::collections::HashMap;


// Todo: Write this hashmap set using arcmutex
fn main() {
    let mut pratice_hash_map = HashMap::new();

    pratice_hash_map.insert("Hello", "120293930");
    pratice_hash_map.insert("Hello2", "120293930");
    pratice_hash_map.insert("Hello3", "120293930");
    pratice_hash_map.insert("Hello4", "120293930");
    pratice_hash_map.insert("Hello5", "120293930");

    println!("{:?}", pratice_hash_map);
}
