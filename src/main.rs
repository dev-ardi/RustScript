use std::{env, io};

use anyhow::anyhow;

#[derive(Debug, Clone, Copy, Hash)]
enum Token<'a> {}
#[derive(Debug, Clone, Copy, Hash)]
enum TokenOwning {}

fn lexer<'a>(source: &'a str) -> anyhow::Result<impl Iterator<Item = Token<'a>>> {}

fn main() -> anyhow::Result<()> {
    match env::args().nth(2) {
        Some(&"-h") | Some(&"--help") => {
            println!("Welcome to RustScript. You can either run a source file with rsc path/to/file or just type rsc to enter the REPL");
            Ok(())
        }
        Some(source_file) => {
            let source = std::io::read_to_string(source_file)?;
            let lexed = lexer(source.as_str());

            Ok(())
        }
        // REPL
        None => {
            let mut line = String::new();
            loop {
                print!("> ");

                if let Ok(0) = io::stdin().read_line(&mut line)? {
                    Ok(()) // Control D
                }

                if let Err(e) = run(line.as_str()) {
                    println!("{:?}", e)
                }
            }
        }
    }
}

fn run(line: &str) -> anyhow::Result<()> {
    Ok(())
}
