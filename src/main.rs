use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, stdout, Write};
use std::process::{Command, Output, Stdio};
use std::{env, str};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("$ ");
        stdout.flush().unwrap();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let (command, stdout_path) = parse_command(input);
        let command: Vec<&str> = command.iter().map(|x| &**x).collect();
        match command.as_slice() {
            [] => continue,
            [""] => continue,
            ["exit", args @ ..] => exit_cmd(args),
            ["echo", args @ ..] => echo_cmd(args, resolve_stdout(stdout_path)),
            ["type", args @ ..] => type_cmd(args, resolve_stdout(stdout_path)),
            ["pwd"] => pwd_cmd(resolve_stdout(stdout_path)),
            ["cd", args @ ..] => cd_cmd(args),
            [command, args @ ..] => try_not_builtin_command(command, args, resolve_stdout(stdout_path)),
        }
    }
}

fn parse_command(input: String) -> (Vec<String>, Option<String>) {
    let input = input.trim().to_string();
    let mut command = vec![];
    let mut curr = String::new();
    let mut single_quoting = false;
    let mut double_quoting = false;
    let mut backlash = false;
    let mut reading_stdout_path = false;
    let mut stdout_path: Option<String> = None;
    for char in input.chars() {
        match char {
            ' ' => {
                if single_quoting || double_quoting {
                    curr = curr + " ";
                } else if backlash {
                    backlash = false;
                    curr = curr + " ";
                } else if curr.len() > 0 {
                    command.push(curr.clone());
                    curr.clear();
                }
            }
            '\'' => {
                if double_quoting && backlash {
                    curr = curr + "\\" + "'"
                } else if double_quoting || backlash {
                    curr = curr + "'";
                } else {
                    single_quoting = !single_quoting;
                }
                backlash = false;
            }
            '\"' => {
                if backlash || single_quoting {
                    curr = curr + "\"";
                } else {
                    double_quoting = !double_quoting;
                }
                backlash = false;
            }
            '\\' => {
                if (double_quoting && backlash) || single_quoting {
                    backlash = false;
                    curr = curr + "\\";
                } else if double_quoting {
                    backlash = true;
                } else if backlash {
                    backlash = false;
                    curr = curr + "\\";
                } else {
                    backlash = true;
                }
            },
            '>' => {
                reading_stdout_path = true;
            },
            other => {
                if backlash && double_quoting {
                    match other {
                        '\n' => curr = curr + "",
                        not_special => curr = curr + "\\" + &not_special.to_string(),
                    }
                } else if reading_stdout_path {
                    match stdout_path {
                        Some(ref mut path) => {
                            path.push(other);
                        },
                        None => {
                            stdout_path = Some(String::from(other));
                        }
                    }
                } else {
                    curr.push(other);
                }
                backlash = false;
            }
        }
    }
    if curr.len() > 0 {
        command.push(curr);
    }
    return (command, stdout_path);
}

fn exit_cmd(args: &[&str]) {
    let code = if args.len() < 1 {
        1
    } else {
        args[0].parse().unwrap_or(1)
    };
    std::process::exit(code);
}

fn echo_cmd(args: &[&str], mut out: Box<dyn Write>) {
    for arg in args {
        write!(out, "{arg} ").unwrap();
    }
    writeln!(out).unwrap();
}

fn type_cmd(args: &[&str], mut out: Box<dyn std::io::Write>) {
    let builtin = ["exit", "echo", "type", "pwd"];
    for builtin_command in builtin {
        if builtin_command == args[0] {
            writeln!(out, "{builtin_command} is a shell builtin").unwrap();
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
                        writeln!(out, "{file_name} is {path}").unwrap();
                        return;
                    }

                }
            },
            Err(_err) => println!("there was a problem reading directory {} check if the directory exists and rash has valid permissions to read it", path)
        }
    }
    println!("{}: not found", args[0]);
}

fn pwd_cmd(mut out: Box<dyn std::io::Write>) {
    writeln!(out, "{}",  env::current_dir()
        .expect("error: maybe the current directory is deleted or you don't have sufficient persmissions")
        .into_os_string().into_string().unwrap()).unwrap();
}

fn cd_cmd(args: &[&str]) {
    if args.len() > 1 {
        println!("cd: too many arguments");
        return;
    }
    let mut path;
    if args.len() == 0 {
        path = "~".to_string();
    } else {
        path = args[0].to_string();
    }
    if path.starts_with('~') {
        path = path.replace('~', &env::var("HOME").unwrap());
    }
    if env::set_current_dir(&path).is_ok() == false {
        println!("cd: {}: No such file or directory", path);
    }
}

fn try_not_builtin_command(command: &str, args: &[&str], mut out: Box<dyn std::io::Write>) {
    let paths = get_paths();
    for path in paths.iter() {
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry.unwrap();
                    let exec_name = entry.file_name();
                    if exec_name == command {
                        let output = execute_external_program(&exec_name, args);
                        writeln!(out, "{}", String::from_utf8(output.stdout).unwrap()).unwrap();
                        return;
                    }
                }
            }
            Err(_err) => (),
        }
    }

    println!("{command}: command not found");
}

fn execute_external_program(executable_path: &OsStr, args: &[&str]) -> Output {
    let output = Command::new(executable_path)
        .args(args)
        .stdout(Stdio::piped())
        .output()
        .expect("command failed");
    return output;
}

fn get_paths() -> Vec<String> {
    let binding = env::var("PATH").unwrap_or("$PATH".to_string());
    let paths = binding
        .split(':')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    return paths;
}

fn resolve_stdout(out_path: Option<String>) -> Box<dyn std::io::Write> {
    let out: Box<dyn Write>;
    match out_path {
        Some(path) => {
            out = Box::new(File::create(path).expect("couldn't write to file provided"));
        }
        None => out = Box::new(stdout()),
    }
    return out; 
}