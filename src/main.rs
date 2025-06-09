use std::ffi::OsStr;
use std::io::{self, stdout, Write};
use std::{env};
use std::fs;
use std::process::{Command, Stdio};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    loop {
        print!("$ ");
        stdout.flush().unwrap();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let command = parse_command(input);
        let command: Vec<&str> = command.iter().map(|x| &**x).collect();
        match command.as_slice() {
            [] => continue,
            [""] => continue,
            ["exit", args @ ..] => exit_cmd(args),
            ["echo", args @ ..] => echo_cmd(args),
            ["type", args @ ..] => type_cmd(args),
            ["pwd"] => pwd_cmd(),
            ["cd", args @ ..] => cd_cmd(args),
            [command, args @ ..] => try_not_builtin_command(command, args),
        }
    }
}

fn parse_command(input: String) -> Vec<String> {
    let input = input.trim().to_string();
    let mut command = vec![];
    let mut curr = String::new();
    let mut quoting = false;
    for char in input.chars() {
        match char {
            ' ' => {
                if quoting {
                    curr = curr + " ";
                } else if curr.len() > 0 {
                    command.push(curr.clone());
                    curr.clear();
                }
            },
            '\'' => {
                if quoting {
                    quoting = false;
                } else {
                    quoting = true;
                }
            },
            other => {
                curr = curr + &other.to_string();
            }
        }
    }
    if curr.len() > 0 {
        command.push(curr);
    }
    return command;
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
    let builtin = ["exit", "echo", "type", "pwd"];
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

fn pwd_cmd() {
    println!("{}",  env::current_dir()
        .expect("error: maybe the current directory is deleted or you don't have sufficient persmissions")
        .into_os_string().into_string().unwrap());
}

fn cd_cmd(args: &[&str]) {
    if args.len() > 1 {
        println!("cd: too many arguments");
        return;
    }
    let mut path;
    if args.len() == 0 {
        path = "~".to_string();
    }
    else {
        path = args[0].to_string();
    }
    if path.starts_with('~') {
        path = path.replace('~', &env::var("HOME").unwrap());

    }
    if env::set_current_dir(&path).is_ok() == false {
        println!("cd: {}: No such file or directory", path);
    }
}

fn try_not_builtin_command(command: &str, args: &[&str]) {
    
    let paths = get_paths();
    for path in paths.iter() {
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry.unwrap();
                    let exec_name = entry.file_name();
                    if exec_name == command {
                        execute_external_program(&exec_name, args);
                        return;
                    }

                }
            },
            Err(_err) => ()
        }
    }

    println!("{command}: command not found");
}

fn execute_external_program(executable_path: &OsStr, args: &[&str]) {
    let output = Command::new(executable_path)
      .args(args)
      .stdout(Stdio::piped())
      .output()
      .expect("command failed");
    stdout().write_all(&output.stdout).unwrap();
}

fn get_paths() -> Vec<String> {
    let binding = env::var("PATH").unwrap_or("$PATH".to_string());
    let paths = binding
        .split(':')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    return paths;
}