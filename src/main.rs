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
    
    if std::env::args().any(|a| a == "--debug-ast") {
        for (i, expr) in program.iter().enumerate() {
            eprintln!("Expr {}: {:?}", i, expr);
        }
    }

    // Execute
    let mut runtime = runtime::Runtime::new();
    runtime.execute(program)?;

    // Display output
    println!("\n=== STORY OUTPUT ===\n");
    for line in runtime.get_output() {
        println!("{}", line);
    }

    if !runtime.get_options().is_empty() {
        println!("\n--- OPTIONS ---");
        for (text, target) in runtime.get_options() {
            println!("  > {} -> {}", text, target);
        }
    }

    Ok(())
}
