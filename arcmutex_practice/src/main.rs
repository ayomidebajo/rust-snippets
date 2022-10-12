use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;


fn main() {
    let counter = Arc::new(Mutex::new(HashMap::new()));
 let mut handles = vec![];
    for i in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut contact = counter.lock().unwrap();

            contact.insert(i, "new stuff");
        });
        handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

 println!("Result: {:?}", *counter.lock().unwrap());

}

// fn main() {
//     let counter = Arc::new(Mutex::new(0));
//     let mut handles = vec![];

//     for _ in 0..10 {
//         let counter = Arc::clone(&counter);
//         let handle = thread::spawn(move || {
//             let mut num = counter.lock().unwrap();

//             *num += 1;
//             println!("nume = {:?}", num);
//         });
//         handles.push(handle);
//     }

//     for handle in handles {
//         handle.join().unwrap();
//     }

//     println!("Result: {}", *counter.lock().unwrap());

//     // println!("m = {:?}", m);
// }
