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
        
        let (command, _rest) = input.split_once(' ').unwrap_or((input, ""));
        println!("{}: command not found", command)
    }
}
