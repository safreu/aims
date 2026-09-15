pub trait ProgressReporter {
    fn header(&self, message: &str);
    fn step(&self, current: usize, total: usize, message: &str);
    fn detail(&self, message: &str);
    fn phase(&self, name: &str, message: &str);
    fn success(&self, message: &str);
}

pub struct ConsoleReporter;

impl ProgressReporter for ConsoleReporter {
    fn header(&self, message: &str) {
        println!("{message}\n")
    }

    fn step(&self, current: usize, total: usize, message: &str) {
        println!("[{current}/{total}] {message}")
    }
    fn detail(&self, message: &str) {
        println!("      {message}")
    }
    fn phase(&self, name: &str, message: &str) {
        println!("[{name}] {message}")
    }

    fn success(&self, message: &str) {
        println!("\n{message}")
    }
}
