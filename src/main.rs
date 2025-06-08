#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    loop {
        print!("$ ");
        stdout.flush().unwrap();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let mut splitted_line = input.split_whitespace();
        let command = splitted_line.next().unwrap();
        if command == "exit" {
            let state: i32 = splitted_line.next().unwrap().trim().parse().unwrap();
            std::process::exit(state);
        } else if command == "echo" {
            loop {
                let next_param = splitted_line.next();
                if next_param == None {
                    break;
                }
                print!("{} ", next_param.unwrap());
            }
            println!("\u{8}");
        } else {
            println!("{}: command not found", command);
        }
    }
}
