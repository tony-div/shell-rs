use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    loop {
        print!("$ ");
        stdout.flush().unwrap();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let command = input.trim().split(" ").collect::<Vec<&str>>();
        match command.as_slice() {
            [""] => continue,
            ["exit", args @ ..] => exit_cmd(args),
            ["echo", args @ ..] => echo_cmd(args),
            ["type", args @ ..] => type_cmd(args),
            other => println!("{}: command not found", other[0]),
        }
    }
}

fn exit_cmd(args: &[&str]) {
    std::process::exit(args[0].parse().unwrap())
}

fn echo_cmd(args: &[&str]) {
    for arg in args {
        print!("{arg} ");
    }
    println!("\u{8}");
}

fn type_cmd(args: &[&str]) {
    let builtin = ["exit", "echo", "type"];
    for builtin_command in builtin {
        if builtin_command == args[0] {
            println!("{builtin_command} is a shell builtin");
            return;
        }
    }
    println!("{}: not found", args[0]);
}