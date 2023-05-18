#[derive(Clone, Debug)]
struct TaskManager(Vec<Task>);

#[derive(Clone, Debug)]
struct Task {
    description: String,
    completed: bool,
}

// TODO: make tasks come from args in command line

impl TaskManager {
    fn new() -> Self {
        TaskManager(vec![])
    }

    fn add(&mut self, task: Task) {
        self.0.push(task);
    }

    fn get_tasks(self) -> Vec<Task> {
        self.0
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
    let new_task = Task::new("Some task".to_string());
    let mut tasks = TaskManager::new();

    tasks.add(new_task);

    println!("robbing the bank {:?}", tasks.get_tasks());
}
