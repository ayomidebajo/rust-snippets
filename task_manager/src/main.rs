struct TaskManager (Vec<Task>);

struct Task {
    description: String,
    completed: bool,
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
    println!("Hello, world!");
}
