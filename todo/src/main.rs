use std::io;

#[derive(Debug)]
struct Todo {
    todos: Vec<String>,
}

pub fn todo() {
    println!("Please input a todo");
    let mut new_vector = Vec::new();
    let mut new_todo_store = Todo { todos: new_vector };
    loop {
        let mut new_string = String::from("");
        io::stdin().read_line(&mut new_string);

        new_todo_store.todos.push(String::from(new_string.trim()));

        println!("inputted {:?}", new_todo_store)
    }
}
