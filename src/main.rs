mod engine;
mod parser;
mod types;

use engine::DatabaseEngine;
use parser::{parse_sql, SqlCommand};
use std::io::{self, BufRead, Write};

fn main() {
    println!("==================================================");
    println!("       Welcome to MiniSQL Engine in Rust!         ");
    println!("==================================================");
    println!("Type 'HELP;' to see available commands or 'EXIT;'.\n");
    println!("Made with love, by Yan C.\n");

    let engine = DatabaseEngine::new("./data");
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    loop {
        print!("minisql> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if reader.read_line(&mut input).is_err() {
            println!("Error reading input.");
            break;
        }

        let cmd_str = input.trim();
        if cmd_str.is_empty() {
            continue;
        }

        match parse_sql(cmd_str) {
            Ok(SqlCommand::Exit) => {
                println!("Goodbye!");
                break;
            }
            Ok(cmd) => match engine.execute(cmd) {
                Ok(result) => println!("{}\n", result),
                Err(err) => println!("Error: {}\n", err),
            },
            Err(e) => println!("Syntax Error: {}\n", e),
        }
    }
}