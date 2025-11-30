use packard::lexer;

fn main() {
    let source = std::fs::read_to_string("debug_lexer.psl").unwrap();
    match lexer::tokenize(&source) {
        Ok(tokens) => {
            for (i, token) in tokens.iter().enumerate() {
                println!("{}: {:?}", i, token);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
