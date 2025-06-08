#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    // Wait for user input
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();
    println!("{}: command not found", input.trim());
}
