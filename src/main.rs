use std::io::{self, Write};

fn main() {
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
        let params = params.trim();
        match command {
            "exit" => process_exit(params),
            _ => eprintln!("{command}: command not found")
        }
    }
}

fn split_input(input: &str) -> (&str, &str) {
    let (head, tail) = input.split_once(' ').unwrap_or((input, ""));
    (head, tail.trim())
}

fn process_exit(params: &str) {
    let (exit_code, _) = split_input(params);
    let exit_code = if exit_code.len() == 0 {
        0
    } else {
        let Ok(res) = exit_code.parse() else {
            eprintln!("exit: {exit_code}: invalid integer");
            return;
        };
        res
    };
    std::process::exit(exit_code);
}
