use std::env;
use std::fs;
use std::process;

use packard::game_engine::GameEngine;
use packard::error::PslError;

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

    match run_interactive(&source, filename) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    }
}

fn run_interactive(source: &str, filename: &str) -> Result<(), PslError> {
    let mut engine = GameEngine::new(source, filename)?;
    engine.run_interactive()?;
    Ok(())
}

#[allow(dead_code)]
fn run_interactive_impl(source: &str, filename: &str) -> Result<(), PslError> {
    run_interactive(source, filename)
}
