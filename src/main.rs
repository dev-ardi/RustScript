use anyhow::anyhow;
use lexer::lex;
mod lexer;
use std::{env, io};

fn main() -> anyhow::Result<()> {
    match env::args().nth(2).as_deref() {
        Some("-h") | Some("--help") => {
            println!("Welcome to RustScript. You can either run a source file with rsc path/to/file or just type rsc to enter the REPL");
            Ok(())
        }
        Some(source_file) => {
            let source = std::fs::read_to_string(source_file)?;
            let lexed = lex(source.as_str());

            Ok(())
        }
        // REPL
        None => {
            let mut line = String::new();
            loop {
                print!("> ");

                if io::stdin().read_line(&mut line)? == 0 {
                    return Ok(()); // Control D
                }

                if let Err(e) = run(line.as_str()) {
                    println!("{:?}", e)
                }
            }
        }
    }
}

fn run(line: &str) -> anyhow::Result<()> {
    let lexed = lex(line);
    Ok(())
}
