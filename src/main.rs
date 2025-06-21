use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::{env, str};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("$ ");
        stdout.flush().unwrap();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let (command, stdout_path, stderr_path) = parse_command(input);
        let command: Vec<&str> = command.iter().map(|x| &**x).collect();
        let out_dest = match resolve_out(stdout_path.as_ref()) {
            Some(file) => {
                match file {
                    Ok(handle) => Some(handle),
                    Err(_err) => {
                        println!("{}: {}: No such file or directory", command[0], stdout_path.unwrap_or(String::new()));
                        continue; 
                    }

                }
            },
            None => None
        };

        let err_dest = match resolve_out(stderr_path.as_ref()) {
            Some(file) => {
                match file {
                    Ok(handle) => Some(handle),
                    Err(_err) => {
                        println!("{}: {}: No such file or directory", command[0], stderr_path.unwrap_or(String::new()));
                        continue; 
                    }

                }
            },
            None => None
        };

        match command.as_slice() {
            [] => continue,
            [""] => continue,
            ["exit", args @ ..] => exit_cmd(args),
            ["echo", args @ ..] => echo_cmd(args, out_dest),
            ["type", args @ ..] => type_cmd(args, out_dest, err_dest),
            ["pwd"] => pwd_cmd(out_dest, err_dest),
            ["cd", args @ ..] => cd_cmd(args),
            [command, args @ ..] => try_not_builtin_command(command, args, out_dest, err_dest),
        }
    }
}

fn parse_command(input: String) -> (Vec<String>, Option<String>, Option<String>) {
    let input = input.trim().to_string();
    let mut command = vec![];
    let mut curr = String::new();
    let mut single_quoting = false;
    let mut double_quoting = false;
    let mut backlash = false;
    let mut reading_stdout_path = false;
    let mut stdout_path: Option<String> = None;
    let mut reading_stderr_path = false;
    let mut stderr_path: Option<String> = None;
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
            },
            '\'' => {
                if double_quoting && backlash {
                    curr = curr + "\\" + "'"
                } else if double_quoting || backlash {
                    curr = curr + "'";
                } else {
                    single_quoting = !single_quoting;
                }
                backlash = false;
            },
            '\"' => {
                if backlash || single_quoting {
                    curr = curr + "\"";
                } else {
                    double_quoting = !double_quoting;
                }
                backlash = false;
            },
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
                reading_stdout_path = curr == "" || curr == "1";
                reading_stderr_path = curr == "2";
                curr.clear();
            },
            other => {
                if backlash && double_quoting {
                    match other {
                        '\n' => curr = curr + "",
                        not_special => curr = curr + "\\" + &not_special.to_string()
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
                } else if reading_stderr_path {
                    match stderr_path {
                        Some(ref mut path) => {
                            path.push(other);
                        },
                        None => {
                            stderr_path = Some(String::from(other));
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
    return (command, stdout_path, stderr_path);
}

fn exit_cmd(args: &[&str]) {
    let code = if args.len() < 1 { 1 } else { args[0].parse().unwrap_or(1) };
    std::process::exit(code);
}

fn echo_cmd(args: &[&str], mut out: Option<File>) {
    for arg in args {
        match out {
            Some(ref mut handle) => {
                write!(handle, "{arg} ").unwrap();
            },
            None => {
                print!("{arg} ");
            }
        }
    }
    match out {
        Some(ref mut handle) => {
            writeln!(handle).unwrap();
        },
        None => {
            println!();
        }
    }
}

fn type_cmd(args: &[&str], mut out: Option<File>, mut err: Option<File>) {
    let builtin = ["exit", "echo", "type", "pwd"];
    for builtin_command in builtin {
        if builtin_command == args[0] {
            match out {
                Some(ref mut handle) => {
                    writeln!(handle, "{builtin_command} is a shell builtin").unwrap();
                },
                None => println!("{builtin_command} is a shell builtin")
            }
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
                        match out {
                            Some(ref mut handle) => {
                                writeln!(handle, "{file_name} is {path}").unwrap();
                            },
                            None => println!("{file_name} is {path}")
                        }
                        return;
                    }

                }
            },
            Err(_err) => {
                match err {
                    Some(ref mut handle) => {
                        writeln!(handle, "there was a problem reading directory {} check if the directory exists and rash has valid permissions to read it", path).unwrap()
                    },
                    None => println!("there was a problem reading directory {} check if the directory exists and rash has valid permissions to read it", path)
                }
            },
        }
    }
    println!("{}: not found", args[0]);
}

fn pwd_cmd(mut out: Option<File>, mut err: Option<File>) {
    match env::current_dir() {
        Ok(current_dir) => {
            match out {
                Some(ref mut handle) => {
                    writeln!(handle, "{}",  current_dir.into_os_string().into_string().unwrap()).unwrap();
                },
                None => println!( "{}",  current_dir.into_os_string().into_string().unwrap())
            }
        },
        Err(_) => 
        match err {
            Some(ref mut handle) => writeln!(handle, "error: maybe the current directory is deleted or you don't have sufficient persmissions").unwrap(),
            None => println!("error: maybe the current directory is deleted or you don't have sufficient persmissions")
        }
    }
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

fn try_not_builtin_command(command: &str, args: &[&str], out: Option<File>, mut err: Option<File>) {
    let paths = get_paths();
    for path in paths.iter() {
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry.unwrap();
                    let exec_name = entry.file_name();
                    if exec_name == command {
                        // let mut child= 
                        execute_external_program(&exec_name, args, out, err);
                        // let stdout_buf= &mut [0u8; 4096];
                        // let stderr_buf= &mut [0u8; 4096];
                        // loop {
                        //     match child.try_wait() {
                        //         Ok(Some(_status)) => { 
                        //             match child.stdout {
                        //                 Some(ref mut child_stdout) => {
                        //                     loop {
                        //                         match child_stdout.read(stdout_buf) {
                        //                             Ok(n) => {
                        //                                 if n == 0 {
                        //                                     break;
                        //                                 }
                        //                                 err.write(&stdout_buf[..n]).unwrap();
                        //                             },
                        //                             Err(_err) => {}
                        //                         }
                        //                     }
                        //                 },
                        //                 None => {}
                        //             }
                        //             match child.stderr {
                        //                 Some(ref mut child_stderr) => {
                        //                     loop {
                        //                         match child_stderr.read(stderr_buf) {
                        //                             Ok(n) => {
                        //                                 if n == 0 {
                        //                                     break;
                        //                                 }
                        //                                 err.write(&stderr_buf[..n]).unwrap();
                        //                             },
                        //                             Err(_err) => {}
                        //                         }
                        //                     }
                        //                 },
                        //                 None => {}
                        //             }
                        //             break;
                        //         },
                        //         Ok(_) => {
                        //             match child.stdout {
                        //                 Some(ref mut child_stdout) => {
                        //                     child_stdout.read(stdout_buf).unwrap();
                        //                     out.write_all(stdout_buf).unwrap();
                        //                 },
                        //                 None => continue
                        //             }
                        //         },
                        //         Err(_) => write!(err, "error while waiting for executable to finish").unwrap()
                        //     }
                        // }
                        return;
                    }
                }
            },
            Err(_err) => ()
        }
    }

    match err {
        Some(ref mut handle) => writeln!(handle, "{command}: command not found").unwrap(),
        None => println!("{command}: command not found")
    }
}

fn execute_external_program(executable_path: &OsStr, args: &[&str], out_file: Option<File>, err_file: Option<File>) {
    let mut command = Command::new(executable_path);
    command.args(args);
        // .stdout(Stdio::from(out.unwrap()))
        // .stderr(Stdio::piped())
        // .spawn()
        // .unwrap();
    match out_file {
        Some(file) => {
            command.stdout(Stdio::from(file));
        },
        None => {}
    }

    match err_file {
        Some(file) => {
            command.stderr(Stdio::from(file));
        },
        None => {}
    }
    command.spawn().unwrap();
}

fn get_paths() -> Vec<String> {
    let binding = env::var("PATH").unwrap_or("$PATH".to_string());
    let paths = binding
        .split(':')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    return paths;
}

fn resolve_out(out_path: Option<&String>) -> Option<io::Result<File>> {
    match out_path {
        Some(path) => {
            return Some(File::create(path));
        }
        None => return None,
    }
}