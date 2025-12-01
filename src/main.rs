use std::env;
use std::fs;
use std::process;

mod lexer;
mod parser;
mod value;
mod runtime;
mod error;

use error::PslError;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file.psl>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    };

    match run(&source) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    }
}

fn run(source: &str) -> Result<(), PslError> {
    // Tokenize
    let tokens = lexer::tokenize(source)?;
    
    if std::env::args().any(|a| a == "--debug-tokens") {
        for (i, token) in tokens.iter().enumerate() {
            eprintln!("{}: {:?}", i, token);
        }
    }

    // Parse
    let program = parser::parse(tokens)?;

    // Execute
    let mut runtime = runtime::Runtime::new();
    runtime.execute(program)?;

    Ok(())
}
