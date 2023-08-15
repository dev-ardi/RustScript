use std::{env, io};

use anyhow::anyhow;

#[derive(Debug, Clone, Copy, Hash)]
enum Token<'a> {}
#[derive(Debug, Clone, Copy, Hash)]
enum TokenOwning {}

fn lexer<'a>(source: &'a str) -> impl Iterator<Item = Token<'a>> {}

fn main() -> anyhow::Result<()> {
    match env::args().nth(2) {
        Some(source_file) => {
            let source = std::io::read_to_string(source_file)?;

            for line in source.lines() {
                run(line)?;
            }
            return Ok(());
        }
        None => {
            let mut line = String::new();
            loop {
                print!("> ");
                io::stdin().read_line(&mut line)?;
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
