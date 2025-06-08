use std::io::{self, Write};
use std::env;
use std::fs;

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
    let code = if args.len() < 1 { 1 } else { args[0].parse().unwrap_or(1) };
    std::process::exit(code);
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
    let paths = get_paths();
    for path in paths.iter() {
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry.unwrap();
                    let path = entry.path().into_os_string().into_string().unwrap();
                    let file_name = entry.file_name().into_string().unwrap();
                    if file_name == args[0] {
                        println!("{file_name} is {path}");
                        return;
                    }

                }
            },
            Err(_err) => println!("there was a problem reading directory {} check if the directory exists and rash has valid permissions to read it", path)
        }
    }
    println!("{}: not found", args[0]);
}

fn get_paths() -> Vec<String> {
    let binding = env::var("PATH").unwrap_or("$PATH".to_string());
    let paths = binding
        .split(':')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    return paths;
}