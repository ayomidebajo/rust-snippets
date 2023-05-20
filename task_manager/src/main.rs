use std::collections::HashMap;
use std::io;

#[derive(Clone, Debug)]
struct TaskManager(HashMap<u32, Task>);

#[derive(Clone, Debug)]
struct Task {
    description: String,
    completed: bool,
}

impl TaskManager {
    fn new() -> Self {
        let new_manager: HashMap<u32, Task> = HashMap::new();
        TaskManager(new_manager)
    }

    fn add(&mut self, task: Task) {
        let count = self.0.len() as u32;
        self.0.insert(count, task);
    }

    fn get_tasks(&self) -> &HashMap<u32, Task> {
        &self.0
    }
}

impl Task {
    fn new(description: String) -> Self {
        Task {
            description,
            completed: false,
        }
    }
}

impl Completed for Task {
    fn completed(&self) -> bool {
        self.completed
    }

    fn set_status(&mut self, completed: bool) {
        self.completed = completed;
    }

    fn toggle_status(&mut self) {
        self.completed = !self.completed
    }
}

trait Completed {
    fn completed(&self) -> bool;
    fn set_status(&mut self, completed: bool);
    fn toggle_status(&mut self);
}

fn main() {
    let mut tasks = TaskManager::new();
    loop {
        println!("Welcome to your task manager");

        println!("Please enter your task");

        let mut task_string = String::from("");
        let stdin = io::stdin();

        match stdin.read_line(&mut task_string) {
            Ok(_) => {
                println!("{:?}", task_string.trim());
            }
            Err(err) => println!("{:?}", err),
        }

        let task_string = task_string.trim();

        let mut new_task = Task::new(task_string.to_string());
        new_task.set_status(true);
        tasks.add(new_task);

        println!("Total tasks {:?}", tasks.get_tasks());
        for task in tasks.get_tasks() {
            println!(
                "Task: {:?} completed {:?}",
                task.1.description, task.1.completed
            );
        }
    }
}
