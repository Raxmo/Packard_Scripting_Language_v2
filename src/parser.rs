use crate::error::PslError;
use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    Text(String),
    Number(f64),
    Flag(bool),
    Item,
    
    // Operations
    Define {
        target: Box<Expr>,
        body: Box<Expr>,
    },
    Set {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Add {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Remove {
        target: Box<Expr>,
    },
    From {
        base: Box<Expr>,
        path: Box<Expr>,
    },
    If {
        condition: Box<Expr>,
        body: Box<Expr>,
    },
    Attribute(String),
    Character(String),
    Container(String),
    Chapter(String),
    Section(String),
    
    // Complex expressions
    Keyword {
        name: String,
        params: Vec<(String, Expr)>,
    },
    SequenceExpr(Vec<Expr>),
    ComparisonOp {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    ArithmeticOp {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current_token(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn peek_token(&self) -> &Token {
        self.tokens.get(self.pos + 1).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), PslError> {
        if self.current_token() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(PslError::ParseError {
                line: 0,
                col: 0,
                message: format!("Expected {:?}, found {:?}", expected, self.current_token()),
            })
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Expr>, PslError> {
        let mut expressions = Vec::new();

        while self.current_token() != &Token::Eof {
            expressions.push(self.parse_expr()?);
        }

        Ok(expressions)
    }

    fn parse_expr(&mut self) -> Result<Expr, PslError> {
        self.parse_comparison_expr()
    }

    fn parse_comparison_expr(&mut self) -> Result<Expr, PslError> {
        let mut left = self.parse_primary_expr()?;

        // Check for comparison operators
        loop {
            let op = match self.current_token() {
                Token::Gt => ">",
                Token::Lt => "<",
                Token::GtEq => ">=",
                Token::LtEq => "<=",
                Token::EqEq => "==",
                Token::NotEq => "!=",
                Token::Plus => "+",
                Token::Minus => "-",
                Token::Star => "*",
                Token::Slash => "/",
                _ => break,
            };

            self.advance();
            let right = self.parse_primary_expr()?;

            left = if matches!(op, ">" | "<" | ">=" | "<=" | "==" | "!=") {
                Expr::ComparisonOp {
                    op: op.to_string(),
                    left: Box::new(left),
                    right: Box::new(right),
                }
            } else {
                Expr::ArithmeticOp {
                    op: op.to_string(),
                    left: Box::new(left),
                    right: Box::new(right),
                }
            };
        }

        Ok(left)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, PslError> {
        match self.current_token() {
            Token::LBracket => self.parse_bracketed_expr(),
            Token::Atom(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Text(name))
            }
            Token::Comma => {
                // Skip trailing/leading commas
                self.advance();
                self.parse_primary_expr()
            }
            other => Err(PslError::ParseError {
                line: 0,
                col: 0,
                message: format!("Unexpected token: {:?}", other),
            }),
        }
    }

    fn parse_bracketed_expr(&mut self) -> Result<Expr, PslError> {
        self.expect(Token::LBracket)?;

        // Check for double-bracketed expression like [[keyword: ...]: body]
        // This happens when first token is also LBracket
        if self.current_token() == &Token::LBracket {
            // Peek ahead to see the keyword and check if this is a statement form
            let peek_pos = self.pos + 1;
            let (is_statement_form, keyword_str) = if let Some(Token::Atom(kw)) = self.tokens.get(peek_pos) {
                let is_stmt = matches!(kw.as_str(),
                    "define" | "set" | "add" | "chapter" | "display" | "if" | "as" | "option" | "from"
                );
                (is_stmt, kw.clone())
            } else {
                (false, String::new())
            };

            if !is_statement_form {
                // Not a statement form, parse the full expression (which may include binary operators)
                let inner_expr = self.parse_comparison_expr()?;
                self.expect(Token::RBracket)?;
                return Ok(inner_expr);
            }

            // For statement forms, check if this is JUST a wrapped expression or a full statement
            // by looking for the pattern [[kw: target]: body]
            // If we find the colon after the target closes, it's a statement
            // We'll parse the inner [kw: ...] and then check what comes after
            self.expect(Token::LBracket)?;
            
            let keyword = keyword_str;
            self.expect(Token::Atom(keyword.clone()))?;
            self.expect(Token::Colon)?;

            // Parse the target/argument for this statement
            // Some keywords (like 'as') might have an empty target (just a closing bracket)
            let target = if self.current_token() == &Token::RBracket && matches!(keyword.as_str(), "as") {
                // Empty target for 'as' statement - use a placeholder
                Expr::Item
            } else {
                match keyword.as_str() {
                    "define" => self.parse_expr()?,
                    "set" => self.parse_expr()?,
                    "add" => self.parse_expr()?,
                    "from" => {
                        let base = self.parse_expr()?;
                        base
                    },
                    "chapter" => {
                        let name = self.parse_until_bracket()?;
                        Expr::Chapter(name)
                    }
                    "display" => self.parse_expr()?,
                    "if" => self.parse_expr()?,
                    "as" => self.parse_expr()?,
                    "option" => self.parse_expr()?,
                    _ => self.parse_expr()?,
                }
            };

            
            // For statement forms like [[keyword: target]: body], after parsing the target,
            // we're positioned right after the target expression.
            // The next token should be either:
            // - Colon: this is a full statement with body: [[kw: target]: body]
            // - RBracket: the target was wrapped in brackets but no body, close and return
            // 
            // But we need to be careful: the target might be a complete statement form itself
            // (like [[from:...]: ...]), which means we're already past the target's closing brackets
            // In that case, we just check for the colon that indicates a body
            
            // First, check if we need to consume a closing bracket for the [keyword: target] part
            // If the target was NOT a statement form, there will be an RBracket here
            if self.current_token() == &Token::RBracket {
                self.advance(); // consume it
            }
            // If the target WAS a statement form, we're already past its closing brackets,
            // so we don't need to consume anything
            
            if self.current_token() == &Token::Colon {
                // This is a full statement form: [[kw: target]: body]
                self.advance(); // consume colon
                
                // Parse body expressions until final RBracket
                let mut body_exprs = Vec::new();
                while self.current_token() != &Token::RBracket && self.current_token() != &Token::Eof {
                    body_exprs.push(self.parse_expr()?);
                    if self.current_token() == &Token::Comma {
                        self.advance();
                    }
                }

                self.expect(Token::RBracket)?;

                let body = if body_exprs.len() == 1 {
                    Box::new(body_exprs.into_iter().next().unwrap())
                } else {
                    Box::new(Expr::SequenceExpr(body_exprs))
                };

                // Reconstruct the appropriate expression based on keyword
                match keyword.as_str() {
                    "define" => {
                        return Ok(Expr::Define {
                            target: Box::new(target),
                            body,
                        })
                    }
                    "set" => {
                        return Ok(Expr::Set {
                            target: Box::new(target),
                            value: body,
                        })
                    }
                    "add" => {
                        return Ok(Expr::Add {
                            target: Box::new(target),
                            value: body,
                        })
                    }
                    "from" => {
                        return Ok(Expr::From {
                            base: Box::new(target),
                            path: body,
                        })
                    }
                    "if" => {
                        return Ok(Expr::If {
                            condition: Box::new(target),
                            body,
                        })
                    }
                    "chapter" => {
                        // Store chapter and display text
                        return Ok(Expr::Keyword {
                            name: "chapter".to_string(),
                            params: vec![
                                ("id".to_string(), target),
                                ("body".to_string(), body.as_ref().clone()),
                            ],
                        })
                    }
                    _ => {
                        return Ok(Expr::Keyword {
                            name: keyword,
                            params: vec![("body".to_string(), body.as_ref().clone())],
                        })
                    }
                }
            } else {
                // No colon after the target, so this is just a wrapped expression
                // Close the outer bracket and return the target
                self.expect(Token::RBracket)?;
                return Ok(target);
            }
        }

        // Single-bracketed expression [keyword: ...]
        let keyword = match self.current_token() {
            Token::Atom(name) => name.clone(),
            Token::Colon => {
                // This might be [: ...] for unnamed expression
                self.expect(Token::Colon)?;
                return self.parse_keyword_params("".to_string());
            }
            other => {
                return Err(PslError::ParseError {
                    line: 0,
                    col: 0,
                    message: format!("Expected keyword, found {:?}", other),
                })
            }
        };
        self.advance();

        self.expect(Token::Colon)?;

        // Now parse based on the keyword
        let expr = match keyword.as_str() {
            "text" => {
                let content = self.parse_until_bracket()?;
                Expr::Text(content)
            }
            "number" => {
                let num_str = self.parse_until_bracket()?;
                match num_str.parse::<f64>() {
                    Ok(n) => Expr::Number(n),
                    Err(_) => {
                        return Err(PslError::ParseError {
                            line: 0,
                            col: 0,
                            message: format!("Invalid number: {}", num_str),
                        })
                    }
                }
            }
            "flag" => {
                let flag_str = self.parse_until_bracket()?;
                let value = match flag_str.as_str() {
                    "on" => true,
                    "off" => false,
                    _ => {
                        return Err(PslError::ParseError {
                            line: 0,
                            col: 0,
                            message: format!("Invalid flag value: {}. Use 'on' or 'off'", flag_str),
                        })
                    }
                };
                Expr::Flag(value)
            }
            "item" => Expr::Item,
            "from" => self.parse_from_expr()?,
            "remove" => self.parse_remove_expr()?,
            "character" => {
                let name = self.parse_until_bracket()?;
                Expr::Character(name)
            }
            "container" => {
                let name = self.parse_until_bracket()?;
                Expr::Container(name)
            }
            "attribute" => {
                let name = self.parse_until_bracket()?;
                Expr::Attribute(name)
            }
            "chapter" => {
                let name = self.parse_until_bracket()?;
                Expr::Chapter(name)
            }
            "section" => {
                let name = self.parse_until_bracket()?;
                Expr::Section(name)
            }
            "goto" => {
                // goto can take chapter: ... or section: ...
                let target = self.parse_expr()?;
                Expr::Keyword {
                    name: "goto".to_string(),
                    params: vec![("target".to_string(), target)],
                }
            }
            "exists" => {
                // Check if something exists: [exists: [from: ...]]
                let target = self.parse_expr()?;
                Expr::Keyword {
                    name: "exists".to_string(),
                    params: vec![("target".to_string(), target)],
                }
            }
            "define" | "set" | "add" => {
                // These should be handled via double-bracket syntax
                return Err(PslError::ParseError {
                    line: 0,
                    col: 0,
                    message: format!("{} requires double-bracket syntax [[{}:...]: body]", keyword, keyword),
                });
            }
            _ => self.parse_keyword_params(keyword)?,
        };

        self.expect(Token::RBracket)?;
        Ok(expr)
    }

    fn parse_until_bracket(&mut self) -> Result<String, PslError> {
        let mut content = String::new();

        while self.current_token() != &Token::RBracket && self.current_token() != &Token::Eof {
            match self.current_token() {
                Token::Atom(s) => {
                    if !content.is_empty() {
                        content.push(' ');
                    }
                    content.push_str(s);
                    self.advance();
                }
                Token::Comma => break,
                Token::LBracket => {
                    // Nested expression, need to handle it
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }

        Ok(content.trim().to_string())
    }

    fn parse_keyword_params(&mut self, keyword: String) -> Result<Expr, PslError> {
        let mut params = Vec::new();

        while self.current_token() != &Token::RBracket && self.current_token() != &Token::Eof {
            // Parse param: expr format
            if let Token::Atom(key) = self.current_token() {
                let key = key.clone();
                self.advance();

                if self.current_token() == &Token::Colon {
                    self.advance();
                    let value = self.parse_expr()?;
                    params.push((key, value));

                    if self.current_token() == &Token::Comma {
                        self.advance();
                    }
                } else {
                    return Err(PslError::ParseError {
                        line: 0,
                        col: 0,
                        message: "Expected ':' after parameter name".to_string(),
                    });
                }
            } else {
                break;
            }
        }

        Ok(Expr::Keyword {
            name: keyword,
            params,
        })
    }

    fn parse_define_expr(&mut self) -> Result<Expr, PslError> {
        let target = self.parse_expr()?;
        self.expect(Token::RBracket)?;

        if self.current_token() == &Token::Colon {
            self.advance();
        }

        let mut body_exprs = Vec::new();
        
        // Collect all following expressions until we hit a closing bracket at our level
        while self.current_token() != &Token::RBracket && self.current_token() != &Token::Eof {
            body_exprs.push(self.parse_expr()?);
            
            if self.current_token() == &Token::Comma {
                self.advance();
            }
        }

        let body = if body_exprs.len() == 1 {
            Box::new(body_exprs.into_iter().next().unwrap())
        } else {
            Box::new(Expr::SequenceExpr(body_exprs))
        };

        Ok(Expr::Define {
            target: Box::new(target),
            body,
        })
    }

    fn parse_set_expr(&mut self) -> Result<Expr, PslError> {
        let target = self.parse_expr()?;
        self.expect(Token::RBracket)?;
        self.expect(Token::Colon)?;
        let value = self.parse_expr()?;

        Ok(Expr::Set {
            target: Box::new(target),
            value: Box::new(value),
        })
    }

    fn parse_add_expr(&mut self) -> Result<Expr, PslError> {
        let target = self.parse_expr()?;
        self.expect(Token::RBracket)?;
        self.expect(Token::Colon)?;
        let value = self.parse_expr()?;

        Ok(Expr::Add {
            target: Box::new(target),
            value: Box::new(value),
        })
    }

    fn parse_remove_expr(&mut self) -> Result<Expr, PslError> {
        let target = self.parse_expr()?;

        Ok(Expr::Remove {
            target: Box::new(target),
        })
    }

    fn parse_from_expr(&mut self) -> Result<Expr, PslError> {
        let base = self.parse_expr()?;
        self.expect(Token::Colon)?;
        let path = self.parse_expr()?;

        Ok(Expr::From {
            base: Box::new(base),
            path: Box::new(path),
        })
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Expr>, PslError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn test_parse_text() {
        let tokens = tokenize("[text: Hello World]").unwrap();
        let exprs = parse(tokens).unwrap();
        assert_eq!(exprs.len(), 1);
        if let Expr::Text(s) = &exprs[0] {
            assert_eq!(s, "Hello World");
        } else {
            panic!("Expected text expression");
        }
    }

    #[test]
    fn test_parse_number() {
        let tokens = tokenize("[number: 42]").unwrap();
        let exprs = parse(tokens).unwrap();
        if let Expr::Number(n) = &exprs[0] {
            assert_eq!(*n, 42.0);
        } else {
            panic!("Expected number expression");
        }
    }
}
