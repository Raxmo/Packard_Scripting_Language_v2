use crate::error::PslError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LBracket,           // [
    RBracket,           // ]
    Colon,              // :
    Comma,              // ,
    Atom(String),       // identifier, keyword, or bare string
    Plus,               // +
    Minus,              // -
    Star,               // *
    Slash,              // /
    Gt,                 // >
    Lt,                 // <
    GtEq,               // >=
    LtEq,               // <=
    EqEq,               // ==
    NotEq,              // !=
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn current_char(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn peek_char(&self) -> Option<char> {
        if self.pos + 1 < self.input.len() {
            Some(self.input[self.pos + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current_char() {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        // Skip // comments
        if self.current_char() == Some('/') && self.peek_char() == Some('/') {
            self.advance();
            self.advance();
            while let Some(ch) = self.current_char() {
                if ch == '\n' {
                    break;
                }
                self.advance();
            }
        }
    }

    fn read_atom(&mut self) -> String {
        let mut atom = String::new();
        
        while let Some(ch) = self.current_char() {
            // Atom continues while we have alphanumeric, underscore, hyphen, or dot
            if ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
                atom.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Special handling: atoms can contain bare string content until bracket
        // For text values like: [text: This is content] the content is the atom
        // This is context-dependent and will be handled by the parser

        atom
    }

    pub fn next_token(&mut self) -> Result<Token, PslError> {
        loop {
            self.skip_whitespace();
            self.skip_comment();
            self.skip_whitespace();

            match self.current_char() {
                None => return Ok(Token::Eof),
                Some('[') => {
                    self.advance();
                    return Ok(Token::LBracket);
                }
                Some(']') => {
                    self.advance();
                    return Ok(Token::RBracket);
                }
                Some(':') => {
                    self.advance();
                    return Ok(Token::Colon);
                }
                Some(',') => {
                    self.advance();
                    return Ok(Token::Comma);
                }
                Some('+') => {
                    self.advance();
                    return Ok(Token::Plus);
                }
                Some('-') => {
                    // Could be minus or part of atom
                    if self.peek_char().map_or(false, |c| c.is_numeric()) {
                        let atom = self.read_atom();
                        return Ok(Token::Atom(format!("-{}", atom)));
                    } else {
                        self.advance();
                        return Ok(Token::Minus);
                    }
                }
                Some('*') => {
                    self.advance();
                    return Ok(Token::Star);
                }
                Some('/') => {
                    if self.peek_char() == Some('/') {
                        self.skip_comment();
                        continue;
                    } else {
                        self.advance();
                        return Ok(Token::Slash);
                    }
                }
                Some('>') => {
                    self.advance();
                    if self.current_char() == Some('=') {
                        self.advance();
                        return Ok(Token::GtEq);
                    }
                    return Ok(Token::Gt);
                }
                Some('<') => {
                    self.advance();
                    if self.current_char() == Some('=') {
                        self.advance();
                        return Ok(Token::LtEq);
                    }
                    return Ok(Token::Lt);
                }
                Some('=') => {
                    self.advance();
                    if self.current_char() == Some('=') {
                        self.advance();
                        return Ok(Token::EqEq);
                    }
                    return Err(PslError::LexError {
                        line: self.line,
                        col: self.col,
                        message: "Unexpected '='. Did you mean '=='?".to_string(),
                    });
                }
                Some('!') => {
                    self.advance();
                    if self.current_char() == Some('=') {
                        self.advance();
                        return Ok(Token::NotEq);
                    }
                    return Err(PslError::LexError {
                        line: self.line,
                        col: self.col,
                        message: "Unexpected '!'. Did you mean '!='?".to_string(),
                    });
                }
                Some(ch) if ch.is_alphanumeric() || ch == '_' => {
                    let atom = self.read_atom();
                    return Ok(Token::Atom(atom));
                }
                Some(ch) => {
                    return Err(PslError::LexError {
                        line: self.line,
                        col: self.col,
                        message: format!("Unexpected character: '{}'", ch),
                    });
                }
            }
        }
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, PslError> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token()?;
        if token == Token::Eof {
            tokens.push(token);
            break;
        }
        tokens.push(token);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let input = "[ atom : number ]";
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens[0], Token::LBracket);
        assert_eq!(tokens[1], Token::Atom("atom".to_string()));
        assert_eq!(tokens[2], Token::Colon);
        assert_eq!(tokens[3], Token::Atom("number".to_string()));
        assert_eq!(tokens[4], Token::RBracket);
    }

    #[test]
    fn test_comments() {
        let input = "[ atom // comment\n : value ]";
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens[0], Token::LBracket);
        assert_eq!(tokens[1], Token::Atom("atom".to_string()));
        assert_eq!(tokens[2], Token::Colon);
        assert_eq!(tokens[3], Token::Atom("value".to_string()));
    }

    #[test]
    fn test_operators() {
        let input = "[ + - * / > < >= <= == != ]";
        let tokens = tokenize(input).unwrap();
        assert!(tokens.iter().any(|t| *t == Token::Plus));
        assert!(tokens.iter().any(|t| *t == Token::Minus));
        assert!(tokens.iter().any(|t| *t == Token::Star));
        assert!(tokens.iter().any(|t| *t == Token::Slash));
    }
}
