use std::collections::HashMap;
use crate::error::PslError;
use crate::parser::Expr;
use crate::value::Value;

pub struct Runtime {
    globals: HashMap<String, Value>,
    current_chapter: Option<String>,
    current_section: Option<String>,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            globals: HashMap::new(),
            current_chapter: None,
            current_section: None,
        }
    }

    pub fn execute(&mut self, program: Vec<Expr>) -> Result<(), PslError> {
        for expr in program {
            self.eval(&expr)?;
        }
        Ok(())
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<Value, PslError> {
        match expr {
            Expr::Text(s) => Ok(Value::Text(s.clone())),
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::Flag(b) => Ok(Value::Flag(*b)),
            Expr::Item => Ok(Value::Item),
            
            Expr::Character(name) => {
                // Return reference to character or error if not found
                self.globals
                    .get(name)
                    .cloned()
                    .ok_or_else(|| PslError::NameError(format!("Undefined character: {}", name)))
            }
            
            Expr::Attribute(name) => {
                // Create an attribute reference
                Ok(Value::Text(format!("[attribute: {}]", name)))
            }
            
            Expr::Container(name) => {
                // Create a container reference
                Ok(Value::Text(format!("[container: {}]", name)))
            }
            
            Expr::Chapter(name) => {
                self.current_chapter = Some(name.clone());
                Ok(Value::Text(name.clone()))
            }
            
            Expr::Section(name) => {
                self.current_section = Some(name.clone());
                Ok(Value::Text(name.clone()))
            }
            
            Expr::Define { target, body } => {
                self.eval_define(target, body)
            }
            
            Expr::Set { target, value } => {
                self.eval_set(target, value)
            }
            
            Expr::Add { target, value } => {
                self.eval_add(target, value)
            }
            
            Expr::Remove { target } => {
                self.eval_remove(target)
            }
            
            Expr::From { base, path } => {
                self.eval_from(base, path)
            }
            
            Expr::SequenceExpr(exprs) => {
                let mut last = Value::Null;
                for e in exprs {
                    last = self.eval(e)?;
                }
                Ok(last)
            }
            
            Expr::Keyword { name, params } => {
                self.eval_keyword(name, params)
            }
            
            Expr::ComparisonOp { op, left, right } => {
                self.eval_comparison(op, left, right)
            }
            
            Expr::ArithmeticOp { op, left, right } => {
                self.eval_arithmetic(op, left, right)
            }
        }
    }

    fn eval_define(&mut self, target: &Expr, body: &Expr) -> Result<Value, PslError> {
        match target {
            Expr::Character(name) => {
                let container = Value::Container(HashMap::new());
                self.globals.insert(name.clone(), container.clone());
                
                // Evaluate body to initialize attributes
                self.eval_in_context(name, body)?;
                
                Ok(container)
            }
            Expr::Container(name) => {
                let container = Value::Container(HashMap::new());
                // Store in parent context (would need parent reference)
                Ok(container)
            }
            _ => Err(PslError::RuntimeError(
                "Invalid define target".to_string(),
            )),
        }
    }

    fn eval_in_context(&mut self, context: &str, expr: &Expr) -> Result<Value, PslError> {
        // This is a simplified version - we need to handle nested contexts properly
        match expr {
            Expr::SequenceExpr(exprs) => {
                let mut last = Value::Null;
                for e in exprs {
                    last = self.eval_in_context(context, e)?;
                }
                Ok(last)
            }
            Expr::Set { target, value } => {
                if let Expr::Attribute(name) = &**target {
                    let val = self.eval(value)?;
                    if let Some(Value::Container(map)) = self.globals.get_mut(context) {
                        map.insert(name.clone(), val.clone());
                        Ok(val)
                    } else {
                        Err(PslError::RuntimeError(format!("Context not found: {}", context)))
                    }
                } else {
                    self.eval(expr)
                }
            }
            Expr::Define { target, body } => {
                if let Expr::Container(name) = &**target {
                    let mut container = Value::Container(HashMap::new());
                    
                    // Add the container to the parent
                    if let Some(Value::Container(map)) = self.globals.get_mut(context) {
                        map.insert(name.clone(), container.clone());
                    }
                    
                    // TODO: Initialize container contents from body
                    Ok(container)
                } else {
                    self.eval(expr)
                }
            }
            _ => self.eval(expr),
        }
    }

    fn eval_set(&mut self, target: &Expr, value: &Expr) -> Result<Value, PslError> {
        let val = self.eval(value)?;
        // TODO: Implement path-based setting
        Ok(val)
    }

    fn eval_add(&mut self, target: &Expr, value: &Expr) -> Result<Value, PslError> {
        let val = self.eval(value)?;
        // TODO: Implement path-based addition
        Ok(val)
    }

    fn eval_remove(&mut self, target: &Expr) -> Result<Value, PslError> {
        // TODO: Implement path-based removal
        Ok(Value::Null)
    }

    fn eval_from(&mut self, base: &Expr, path: &Expr) -> Result<Value, PslError> {
        // This is complex - need to handle nested access
        // For now, return a reference value
        Ok(Value::Text(format!("[from: {:?} : {:?}]", base, path)))
    }

    fn eval_keyword(&mut self, name: &str, _params: &[(String, Expr)]) -> Result<Value, PslError> {
        // Handle special keywords like if, display, option, etc.
        match name {
            "if" => {
                // TODO: Implement conditional logic
                Ok(Value::Null)
            }
            "display" => {
                // TODO: Implement display logic
                Ok(Value::Null)
            }
            "option" => {
                // TODO: Implement option logic
                Ok(Value::Null)
            }
            "as" => {
                // TODO: Implement narrative display
                Ok(Value::Null)
            }
            "goto" => {
                // TODO: Implement navigation
                Ok(Value::Null)
            }
            "exists" => {
                // TODO: Implement existence check
                Ok(Value::Flag(true))
            }
            _ => Err(PslError::RuntimeError(format!(
                "Unknown keyword: {}",
                name
            ))),
        }
    }

    fn eval_comparison(&mut self, op: &str, left: &Expr, right: &Expr) -> Result<Value, PslError> {
        let left_val = self.eval(left)?;
        let right_val = self.eval(right)?;

        let result = match (left_val, right_val) {
            (Value::Number(l), Value::Number(r)) => {
                match op {
                    ">" => l > r,
                    "<" => l < r,
                    ">=" => l >= r,
                    "<=" => l <= r,
                    "==" => (l - r).abs() < f64::EPSILON,
                    "!=" => (l - r).abs() >= f64::EPSILON,
                    _ => return Err(PslError::RuntimeError(format!("Unknown operator: {}", op))),
                }
            }
            _ => return Err(PslError::TypeError(
                "Comparison requires numeric values".to_string(),
            )),
        };

        Ok(Value::Flag(result))
    }

    fn eval_arithmetic(&mut self, op: &str, left: &Expr, right: &Expr) -> Result<Value, PslError> {
        let left_val = self.eval(left)?;
        let right_val = self.eval(right)?;

        match (left_val, right_val) {
            (Value::Number(l), Value::Number(r)) => {
                let result = match op {
                    "+" => l + r,
                    "-" => l - r,
                    "*" => l * r,
                    "/" => {
                        if r == 0.0 {
                            return Err(PslError::RuntimeError("Division by zero".to_string()));
                        }
                        l / r
                    }
                    _ => return Err(PslError::RuntimeError(format!("Unknown operator: {}", op))),
                };
                Ok(Value::Number(result))
            }
            _ => Err(PslError::TypeError(
                "Arithmetic requires numeric values".to_string(),
            )),
        }
    }
}
