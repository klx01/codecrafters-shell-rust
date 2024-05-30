use std::io::{self, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const NOT_FOUND_CODE: i32 = 127;
const TERMINATED_CODE_BASE: i32 = 128;
const EXEC_FAILED_CODE: i32 = 777;

fn main() {
    let mut last_exit_code = 0;
    loop {
        // prompt
        print!("$ ");
        io::stdout().flush().expect("failed to flush prompt");

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).expect("failed to read input");
        if input.len() == 0 {
            // ctrl+d - exit
            break;
        }

        let input = input.trim();
        if input.len() == 0 {
            // ignore empty lines
            continue;
        }

        let (command, params) = split_input(input);
        last_exit_code = match command {
            "exit" => command_exit(params),
            "echo" => command_echo(params, last_exit_code),
            "type" => command_type(params),
            "pwd" => command_pwd(params),
            "cd" => command_cd(params),
            _ => match find_executable(command) {
                Some(path) => execute(&path, params),
                None => {
                    eprintln!("{command}: command not found");
                    NOT_FOUND_CODE
                },
            }
        };
    }
}

fn split_input(input: &str) -> (&str, &str) {
    // todo: needs to handle quoting, backslashes, etc..
    let (head, tail) = input.split_once(' ').unwrap_or((input, ""));
    (head, tail.trim())
}

fn command_exit(params: &str) -> i32 {
    let (exit_code, params) = split_input(params);
    if params.len() > 0 {
        eprintln!("exit: too many arguments");
        return 2;
    }
    let exit_code = if exit_code.len() == 0 {
        0
    } else {
        let Ok(res) = exit_code.parse() else {
            eprintln!("exit: {exit_code}: invalid integer");
            return 2;
        };
        res
    };
    std::process::exit(exit_code);
}

fn command_echo(params: &str, last_exit_code: i32) -> i32 {
    if params == "$0" {
        println!("{last_exit_code}")
    } else {
        println!("{params}");
    }
    0
}

fn command_type(mut params: &str) -> i32 {
    let mut has_success = false;
    loop {
        let (command, tail) = split_input(params);
        params = tail;
        if command.len() == 0 {
            break;
        }
        match command {
            "exit" | "echo" | "type" | "pwd" | "cd" => {
                has_success = true;
                println!("{command} is a shell builtin");
            },
            _ => match find_executable(command) {
                Some(path) => {
                    has_success = true;
                    print!("{command} is ");
                    write_path(path, true);
                },
                None => println!("{command} not found"),
            }
        }
    }
    if has_success {
        0
    } else {
        1
    }
}

fn find_executable(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    for mut path in std::env::split_paths(&paths) {
        path.push(name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

fn write_path(path: PathBuf, add_newline: bool) {
    let _ = io::stdout().write(path.as_os_str().as_encoded_bytes());
    if add_newline {
        print!("\n");
    }
}

fn command_pwd(params: &str) -> i32 {
    if params.len() > 0 {
        eprintln!("pwd: expected 0 arguments");
        return 2;
    }
    let cwd = std::env::current_dir();
    match cwd {
        Ok(cwd) => {
            write_path(cwd, true);
            0
        }
        Err(e) => {
            eprintln!("failed to get pwd {e}");
            2
        }
    }
}

fn command_cd(params: &str) -> i32 {
    if params.len() == 0 {
        // todo: cd to home
        eprintln!("Not enough args for cd command");
        return 1;
    }
    let (to_dir, tail) = split_input(params);
    if tail.len() > 0 {
        eprintln!("Too many args for cd command");
        return 1;
    }
    match std::env::set_current_dir(to_dir) {
        Ok(_) => 0,
        Err(_) => {
            eprintln!("cd: {to_dir}: No such file or directory");
            return 1;
        }
    }
}

fn execute(path: &Path, params: &str) -> i32 {
    let status = Command::new(path)
        .arg(params) // todo: this does not correctly handle multiple arguments
        .status();
    match status {
        Ok(exit_status) => match exit_status.code() {
            Some(code) => code,
            None => match exit_status.signal() {
                Some(signal) => {
                    eprintln!("Process was terminated with signal {signal}");
                    TERMINATED_CODE_BASE + signal
                },
                None => {
                    eprintln!("Process did not return neither code nor termination signal, weird");
                    TERMINATED_CODE_BASE
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to execute, error: {e}; set last exit code to {EXEC_FAILED_CODE}");
            EXEC_FAILED_CODE
        },
    }
}
