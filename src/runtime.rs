use std::collections::HashMap;
use crate::error::PslError;
use crate::parser::Expr;
use crate::value::Value;



pub struct Runtime {
    globals: HashMap<String, Value>,
    current_chapter: Option<String>,
    current_section: Option<String>,
    output: Vec<String>,
    pending_options: Vec<(String, String)>,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            globals: HashMap::new(),
            current_chapter: None,
            current_section: None,
            output: Vec::new(),
            pending_options: Vec::new(),
        }
    }

    pub fn get_output(&self) -> &[String] {
        &self.output
    }

    pub fn get_options(&self) -> &[(String, String)] {
        &self.pending_options
    }

    #[allow(dead_code)]
    pub fn clear_output(&mut self) {
        self.output.clear();
        self.pending_options.clear();
    }

    #[allow(dead_code)]
    pub fn get_globals(&self) -> HashMap<String, Value> {
        self.globals.clone()
    }

    #[allow(dead_code)]
    pub fn get_current_chapter(&self) -> Option<String> {
        self.current_chapter.clone()
    }

    #[allow(dead_code)]
    pub fn get_current_section(&self) -> Option<String> {
        self.current_section.clone()
    }

    #[allow(dead_code)]
    pub fn restore_state(&mut self, 
        globals: HashMap<String, Value>,
        current_chapter: Option<String>,
        current_section: Option<String>,
    ) -> Result<(), PslError> {
        self.globals = globals;
        self.current_chapter = current_chapter;
        self.current_section = current_section;
        Ok(())
    }


    pub fn execute(&mut self, program: Vec<Expr>) -> Result<(), PslError> {
        for (idx, expr) in program.iter().enumerate() {
            match self.eval(expr) {
                Ok(_) => {},
                Err(e) => {
                    return Err(PslError::RuntimeError(
                        format!("Error executing expression {}: {} (expr: {:?})", idx, e, expr)
                    ));
                }
            }
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
                self.globals
                    .get(name)
                    .cloned()
                    .ok_or_else(|| PslError::NameError(format!("Undefined character: {}", name)))
            }
            
            Expr::Attribute(name) => {
                Ok(Value::Text(name.clone()))
            }
            
            Expr::Container(_name) => {
                Ok(Value::Item)
            }
            
            Expr::Chapter(name) => {
                self.current_chapter = Some(name.clone());
                self.output.push(format!("[LOCATION: chapter:{}]", name));
                Ok(Value::Text(name.clone()))
            }
            
            Expr::Section(name) => {
                self.current_section = Some(name.clone());
                self.output.push(format!("[LOCATION: section:{}]", name));
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
            
            Expr::If { condition, body } => {
                self.eval_if_expr(condition, body)
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
                let container = HashMap::new();
                self.globals.insert(name.clone(), Value::Container(container.clone()));
                
                // Evaluate body to initialize attributes
                self.eval_define_body(name.clone(), body)?;
                
                Ok(Value::Container(container))
            }
            Expr::Container(_name) => {
                Ok(Value::Container(HashMap::new()))
            }
            _ => Err(PslError::RuntimeError(
                "Invalid define target".to_string(),
            )),
        }
    }

    // Evaluates the body of a define statement within a character context
    fn eval_define_body(&mut self, character: String, expr: &Expr) -> Result<Value, PslError> {
        match expr {
            Expr::SequenceExpr(exprs) => {
                let mut last = Value::Null;
                for e in exprs {
                    last = self.eval_define_body(character.clone(), e)?;
                }
                Ok(last)
            }
            Expr::Set { target, value } => {
                // Handle [[set: [attribute: name]]: [text: Hero]]
                if let Expr::Attribute(attr_name) = &**target {
                    let val = self.eval(value)?;
                    if let Some(Value::Container(map)) = self.globals.get_mut(&character) {
                        map.insert(attr_name.clone(), val.clone());
                        return Ok(val);
                    } else {
                        return Err(PslError::RuntimeError(format!("Character not found: {}", character)));
                    }
                }
                self.eval(expr)
            }
            Expr::Define { target, body: define_body } => {
                // Handle nested container definitions
                if let Expr::Container(container_name) = &**target {
                    let container_map = HashMap::new();
                    
                    // Add to parent character
                    if let Some(Value::Container(char_map)) = self.globals.get_mut(&character) {
                        char_map.insert(container_name.clone(), Value::Container(container_map.clone()));
                    }
                    
                    // Initialize container contents
                    self.eval_container_body(character.clone(), container_name.clone(), define_body)?;
                    
                    return Ok(Value::Container(container_map));
                }
                self.eval(expr)
            }
            _ => self.eval(expr),
        }
    }

    // Evaluates the body of a container definition
    fn eval_container_body(&mut self, character: String, container: String, expr: &Expr) -> Result<Value, PslError> {
        match expr {
            Expr::SequenceExpr(exprs) => {
                let mut last = Value::Null;
                for e in exprs {
                    last = self.eval_container_body(character.clone(), container.clone(), e)?;
                }
                Ok(last)
            }
            Expr::Set { target, value } => {
                // Handle [[set: [attribute: sword]]: [item:]]
                if let Expr::Attribute(attr_name) = &**target {
                    let val = self.eval(value)?;
                    
                    // Navigate: character -> container -> attribute
                    if let Some(Value::Container(char_map)) = self.globals.get_mut(&character) {
                        if let Some(Value::Container(cont_map)) = char_map.get_mut(&container) {
                            cont_map.insert(attr_name.clone(), val.clone());
                            return Ok(val);
                        } else {
                            return Err(PslError::RuntimeError(
                                format!("Container {} not found in character {}", container, character)
                            ));
                        }
                    } else {
                        return Err(PslError::RuntimeError(format!("Character {} not found", character)));
                    }
                }
                self.eval(expr)
            }
            _ => self.eval(expr),
        }
    }

    fn eval_set(&mut self, target: &Expr, value: &Expr) -> Result<Value, PslError> {
        let val = self.eval(value)?;
        
        // Handle From expressions as targets
        if let Expr::From { base, path } = target {
            return self.set_via_path(base, path, val);
        }
        
        Ok(val)
    }

    fn eval_add(&mut self, target: &Expr, value: &Expr) -> Result<Value, PslError> {
        let val = self.eval(value)?;
        
        // Handle From expressions as targets
        if let Expr::From { base, path } = target {
            // For add, try to get existing value; if it doesn't exist, just set the new value
            let result = match self.get_via_path(base, path) {
                Ok(existing) => {
                    // Type-specific addition
                    match (existing, &val) {
                        (Value::Number(n), Value::Number(m)) => Value::Number(n + m),
                        (Value::Item, Value::Item) => Value::Item,
                        _ => return Err(PslError::TypeError(
                            "Cannot add values of different types".to_string()
                        )),
                    }
                }
                Err(_) => {
                    // If attribute doesn't exist, just use the new value
                    val.clone()
                }
            };
            
            return self.set_via_path(base, path, result);
        }
        
        Ok(val)
    }

    fn eval_remove(&mut self, target: &Expr) -> Result<Value, PslError> {
        // Handle From expressions as targets
        if let Expr::From { base, path } = target {
            return self.remove_via_path(base, path);
        }
        
        Ok(Value::Null)
    }

    // Resolves a path starting from base and following path
    fn eval_from(&mut self, base: &Expr, path: &Expr) -> Result<Value, PslError> {
        self.get_via_path(base, path)
    }

    // Evaluates an if expression: execute body if condition is truthy
    fn eval_if_expr(&mut self, condition: &Expr, body: &Expr) -> Result<Value, PslError> {
        let cond_val = self.eval(condition)?;
        
        // Check if condition is truthy
        if self.is_truthy(&cond_val) {
            // Execute the body
            self.eval(body)
        } else {
            // Don't execute the body, return null
            Ok(Value::Null)
        }
    }

    fn is_truthy(&self, val: &Value) -> bool {
        match val {
            Value::Flag(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::Text(s) => !s.is_empty(),
            Value::Item => true,
            Value::Container(_) => true,
            Value::Null => false,
        }
    }

    // Gets a value by following a path from a base
    fn get_via_path(&mut self, base: &Expr, path: &Expr) -> Result<Value, PslError> {
        // First evaluate the base to get the starting point
        let base_val = self.eval(base)?;
        
        // Now follow the path
        self.follow_path(base_val, path)
    }

    // Follows a path through containers/attributes
    fn follow_path(&mut self, current: Value, path: &Expr) -> Result<Value, PslError> {
        match path {
            Expr::Attribute(name) => {
                // Simple attribute access
                if let Value::Container(map) = current {
                    map.get(name)
                        .cloned()
                        .ok_or_else(|| PslError::RuntimeError(
                            format!("Attribute {} not found", name)
                        ))
                } else {
                    Err(PslError::TypeError(
                        format!("Cannot access attribute on non-container value")
                    ))
                }
            }
            Expr::Container(name) => {
                // Container access
                if let Value::Container(map) = current {
                    map.get(name)
                        .cloned()
                        .ok_or_else(|| PslError::RuntimeError(
                            format!("Container {} not found", name)
                        ))
                } else {
                    Err(PslError::TypeError(
                        format!("Cannot access container on non-container value")
                    ))
                }
            }
            Expr::From { base, path: next_path } => {
                // Nested from chain: [[from: [[from: x]: y]]: z]
                // Follow the inner from first to get intermediate value
                let intermediate = self.follow_path(current, base)?;
                // Then follow the next path
                self.follow_path(intermediate, next_path)
            }
            _ => Err(PslError::RuntimeError(
                "Invalid path expression".to_string()
            )),
        }
    }

    // Sets a value at a path location
    fn set_via_path(&mut self, base: &Expr, path: &Expr, value: Value) -> Result<Value, PslError> {
        // Navigate through globals to find the right location and set it
        self.navigate_and_set(base, path, value)
    }

    fn navigate_and_set(&mut self, base: &Expr, path: &Expr, value: Value) -> Result<Value, PslError> {
        match base {
            Expr::Character(char_name) => {
                // Navigate to character and set via path
                self.set_in_path(char_name.clone(), path, value)
            }
            Expr::From { base: inner_base, path: inner_path } => {
                // Handle nested from by first resolving the inner path
                match inner_base.as_ref() {
                    Expr::Character(char_name) => {
                        // [[from: [character: main]]: [container: bag]]
                        self.set_in_nested_path(char_name.clone(), inner_path, path, value)
                    }
                    _ => Err(PslError::RuntimeError(
                        "Complex nested base not supported".to_string()
                    )),
                }
            }
            _ => Err(PslError::RuntimeError(
                "Invalid base in set path".to_string()
            )),
        }
    }

    fn set_in_path(&mut self, char_name: String, path: &Expr, value: Value) -> Result<Value, PslError> {
        match path {
            Expr::Attribute(attr_name) => {
                // Direct character attribute
                if let Some(Value::Container(map)) = self.globals.get_mut(&char_name) {
                    map.insert(attr_name.clone(), value.clone());
                    Ok(value)
                } else {
                    Err(PslError::RuntimeError(format!("Character {} not found", char_name)))
                }
            }
            Expr::From { base, path: next_path } => {
                // Navigate through base first
                match base.as_ref() {
                    Expr::Container(container_name) => {
                        if let Some(Value::Container(char_map)) = self.globals.get_mut(&char_name) {
                            if let Some(Value::Container(cont_map)) = char_map.get_mut(container_name) {
                                Self::set_in_container(cont_map, next_path, value.clone())?;
                                Ok(value)
                            } else {
                                Err(PslError::RuntimeError(format!("Container {} not found", container_name)))
                            }
                        } else {
                            Err(PslError::RuntimeError(format!("Character {} not found", char_name)))
                        }
                    }
                    _ => Err(PslError::RuntimeError("Invalid path base".to_string())),
                }
            }
            _ => Err(PslError::RuntimeError("Invalid path in set_in_path".to_string())),
        }
    }

    fn set_in_nested_path(
        &mut self,
        char_name: String,
        inner_path: &Expr,
        final_path: &Expr,
        value: Value,
    ) -> Result<Value, PslError> {
        match inner_path {
            Expr::Container(container_name) => {
                // Navigate character -> container -> path
                if let Some(Value::Container(char_map)) = self.globals.get_mut(&char_name) {
                    if let Some(Value::Container(cont_map)) = char_map.get_mut(container_name) {
                        Self::set_in_container(cont_map, final_path, value.clone())?;
                        Ok(value)
                    } else {
                        Err(PslError::RuntimeError(format!("Container {} not found", container_name)))
                    }
                } else {
                    Err(PslError::RuntimeError(format!("Character {} not found", char_name)))
                }
            }
            Expr::From { base, path: next_inner_path } => {
                // Handle nested from
                match base.as_ref() {
                    Expr::Container(container_name) => {
                        if let Some(Value::Container(char_map)) = self.globals.get_mut(&char_name) {
                            if let Some(Value::Container(cont_map)) = char_map.get_mut(container_name) {
                                Self::set_in_nested_container(cont_map, next_inner_path, final_path, value.clone())?;
                                Ok(value)
                            } else {
                                Err(PslError::RuntimeError(format!("Container {} not found", container_name)))
                            }
                        } else {
                            Err(PslError::RuntimeError(format!("Character {} not found", char_name)))
                        }
                    }
                    _ => Err(PslError::RuntimeError("Invalid nested path".to_string())),
                }
            }
            _ => Err(PslError::RuntimeError("Invalid inner path".to_string())),
        }
    }

    fn set_in_container(
        container: &mut HashMap<String, Value>,
        path: &Expr,
        value: Value,
    ) -> Result<(), PslError> {
        match path {
            Expr::Attribute(name) => {
                container.insert(name.clone(), value);
                Ok(())
            }
            Expr::From { base, path: next_path } => {
                match base.as_ref() {
                    Expr::Container(container_name) => {
                        if let Some(Value::Container(inner_map)) = container.get_mut(container_name) {
                            Self::set_in_container(inner_map, next_path, value)?;
                            Ok(())
                        } else {
                            Err(PslError::RuntimeError(format!("Container {} not found", container_name)))
                        }
                    }
                    _ => Err(PslError::RuntimeError("Invalid nested path".to_string())),
                }
            }
            _ => Err(PslError::RuntimeError("Invalid path".to_string())),
        }
    }

    fn set_in_nested_container(
        container: &mut HashMap<String, Value>,
        inner_path: &Expr,
        final_path: &Expr,
        value: Value,
    ) -> Result<(), PslError> {
        match inner_path {
            Expr::Container(container_name) => {
                if let Some(Value::Container(inner_map)) = container.get_mut(container_name) {
                    Self::set_in_container(inner_map, final_path, value)?;
                    Ok(())
                } else {
                    Err(PslError::RuntimeError(format!("Container {} not found", container_name)))
                }
            }
            Expr::From { base, path: next_inner_path } => {
                match base.as_ref() {
                    Expr::Container(container_name) => {
                        if let Some(Value::Container(inner_map)) = container.get_mut(container_name) {
                            Self::set_in_nested_container(inner_map, next_inner_path, final_path, value)?;
                            Ok(())
                        } else {
                            Err(PslError::RuntimeError(format!("Container {} not found", container_name)))
                        }
                    }
                    _ => Err(PslError::RuntimeError("Invalid nested path".to_string())),
                }
            }
            _ => Err(PslError::RuntimeError("Invalid inner path".to_string())),
        }
    }

    // Removes a value at a path location
    fn remove_via_path(&mut self, base: &Expr, path: &Expr) -> Result<Value, PslError> {
        match base {
            Expr::Character(char_name) => {
                self.remove_in_path(char_name.clone(), path)
            }
            Expr::From { base: inner_base, path: inner_path } => {
                match inner_base.as_ref() {
                    Expr::Character(char_name) => {
                        self.remove_in_nested_path(char_name.clone(), inner_path, path)
                    }
                    _ => Err(PslError::RuntimeError(
                        "Complex nested paths not yet supported".to_string()
                    )),
                }
            }
            _ => Err(PslError::RuntimeError(
                "Invalid base in remove path".to_string()
            )),
        }
    }

    fn remove_in_path(&mut self, char_name: String, path: &Expr) -> Result<Value, PslError> {
        match path {
            Expr::Attribute(attr_name) => {
                if let Some(Value::Container(map)) = self.globals.get_mut(&char_name) {
                    map.remove(attr_name)
                        .ok_or_else(|| PslError::RuntimeError(format!("Attribute {} not found", attr_name)))
                } else {
                    Err(PslError::RuntimeError(format!("Character {} not found", char_name)))
                }
            }
            Expr::From { base, path: next_path } => {
                match base.as_ref() {
                    Expr::Container(container_name) => {
                        if let Some(Value::Container(char_map)) = self.globals.get_mut(&char_name) {
                            if let Some(Value::Container(cont_map)) = char_map.get_mut(container_name) {
                                Self::remove_from_container(cont_map, next_path)
                            } else {
                                Err(PslError::RuntimeError(format!("Container {} not found", container_name)))
                            }
                        } else {
                            Err(PslError::RuntimeError(format!("Character {} not found", char_name)))
                        }
                    }
                    _ => Err(PslError::RuntimeError("Invalid path base".to_string())),
                }
            }
            _ => Err(PslError::RuntimeError("Invalid path".to_string())),
        }
    }

    fn remove_in_nested_path(
        &mut self,
        char_name: String,
        inner_path: &Expr,
        final_path: &Expr,
    ) -> Result<Value, PslError> {
        match inner_path {
            Expr::Container(container_name) => {
                if let Some(Value::Container(char_map)) = self.globals.get_mut(&char_name) {
                    if let Some(Value::Container(cont_map)) = char_map.get_mut(container_name) {
                        Self::remove_from_container(cont_map, final_path)
                    } else {
                        Err(PslError::RuntimeError(format!("Container {} not found", container_name)))
                    }
                } else {
                    Err(PslError::RuntimeError(format!("Character {} not found", char_name)))
                }
            }
            Expr::From { base, path: next_inner_path } => {
                match base.as_ref() {
                    Expr::Container(container_name) => {
                        if let Some(Value::Container(char_map)) = self.globals.get_mut(&char_name) {
                            if let Some(Value::Container(cont_map)) = char_map.get_mut(container_name) {
                                Self::remove_from_nested_container(cont_map, next_inner_path, final_path)
                            } else {
                                Err(PslError::RuntimeError(format!("Container {} not found", container_name)))
                            }
                        } else {
                            Err(PslError::RuntimeError(format!("Character {} not found", char_name)))
                        }
                    }
                    _ => Err(PslError::RuntimeError("Invalid nested path".to_string())),
                }
            }
            _ => Err(PslError::RuntimeError("Invalid inner path".to_string())),
        }
    }

    fn remove_from_container(container: &mut HashMap<String, Value>, path: &Expr) -> Result<Value, PslError> {
        match path {
            Expr::Attribute(name) => {
                container.remove(name)
                    .ok_or_else(|| PslError::RuntimeError(format!("Attribute {} not found", name)))
            }
            Expr::From { base, path: next_path } => {
                match base.as_ref() {
                    Expr::Container(container_name) => {
                        if let Some(Value::Container(inner_map)) = container.get_mut(container_name) {
                            Self::remove_from_container(inner_map, next_path)
                        } else {
                            Err(PslError::RuntimeError(format!("Container {} not found", container_name)))
                        }
                    }
                    _ => Err(PslError::RuntimeError("Invalid nested path".to_string())),
                }
            }
            _ => Err(PslError::RuntimeError("Invalid path".to_string())),
        }
    }

    fn remove_from_nested_container(
        container: &mut HashMap<String, Value>,
        inner_path: &Expr,
        final_path: &Expr,
    ) -> Result<Value, PslError> {
        match inner_path {
            Expr::Container(container_name) => {
                if let Some(Value::Container(inner_map)) = container.get_mut(container_name) {
                    Self::remove_from_container(inner_map, final_path)
                } else {
                    Err(PslError::RuntimeError(format!("Container {} not found", container_name)))
                }
            }
            Expr::From { base, path: next_inner_path } => {
                match base.as_ref() {
                    Expr::Container(container_name) => {
                        if let Some(Value::Container(inner_map)) = container.get_mut(container_name) {
                            Self::remove_from_nested_container(inner_map, next_inner_path, final_path)
                        } else {
                            Err(PslError::RuntimeError(format!("Container {} not found", container_name)))
                        }
                    }
                    _ => Err(PslError::RuntimeError("Invalid nested path".to_string())),
                }
            }
            _ => Err(PslError::RuntimeError("Invalid inner path".to_string())),
        }
    }

    fn eval_keyword(&mut self, name: &str, params: &[(String, Expr)]) -> Result<Value, PslError> {
        match name {
            "exists" => self.eval_exists(params),
            "if" => self.eval_if(params),
            "display" => self.eval_display(params),
            "option" => self.eval_option(params),
            "as" => self.eval_as(params),
            "goto" => self.eval_goto(params),
            "chapter" => self.eval_chapter(params),
            _ => Err(PslError::RuntimeError(format!("Unknown keyword: {}", name))),
        }
    }

    fn eval_exists(&mut self, params: &[(String, Expr)]) -> Result<Value, PslError> {
        // Extract the target from params and check if it exists
        for (key, expr) in params {
            if key == "body" {
                match self.eval(expr) {
                    Ok(Value::Null) => return Ok(Value::Flag(false)),
                    Ok(_) => return Ok(Value::Flag(true)),
                    Err(PslError::NameError(_)) | Err(PslError::RuntimeError(_)) => {
                        return Ok(Value::Flag(false))
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(Value::Flag(false))
    }

    fn eval_if(&mut self, params: &[(String, Expr)]) -> Result<Value, PslError> {
        // The if keyword is parsed as [[if: condition]: body] 
        // where condition becomes the target and body is the body
        // But how is this stored in the Keyword? Let me check what we get
        for (key, expr) in params {
            if key == "body" {
                // Evaluate the condition
                let cond_val = self.eval(expr)?;
                // Return the truthy value
                match cond_val {
                    Value::Flag(b) => return Ok(Value::Flag(b)),
                    Value::Number(n) => return Ok(Value::Flag(n != 0.0)),
                    Value::Text(s) => return Ok(Value::Flag(!s.is_empty())),
                    Value::Item => return Ok(Value::Flag(true)),
                    Value::Container(_) => return Ok(Value::Flag(true)),
                    Value::Null => return Ok(Value::Flag(false)),
                }
            }
        }
        Ok(Value::Flag(false))
    }

    fn eval_display(&mut self, params: &[(String, Expr)]) -> Result<Value, PslError> {
        // [[display: text]: options...]
        // Extract display text from params
        for (key, expr) in params {
            if key == "id" {
                let val = self.eval(expr)?;
                if let Value::Text(text) = val {
                    self.output.push(format!("[DISPLAY] {}", text));
                }
            } else if key == "body" {
                // Body contains the options - evaluate but don't display directly
                // Options will register themselves via eval_option
                let _ = self.eval(expr)?;
            }
        }
        Ok(Value::Null)
    }

    fn eval_option(&mut self, params: &[(String, Expr)]) -> Result<Value, PslError> {
        // [[option: text]: goto_target]
        let mut option_text = String::new();
        let mut goto_target = String::new();

        for (key, expr) in params {
            if key == "id" {
                let val = self.eval(expr)?;
                if let Value::Text(text) = val {
                    option_text = text;
                }
            } else if key == "body" {
                // The body should be a goto expression
                match expr {
                    Expr::Keyword { name, params: goto_params } => {
                        if name == "goto" {
                            goto_target = self.eval_goto_target(goto_params)?;
                        }
                    }
                    _ => {
                        // Just evaluate it to see what we get
                        let _ = self.eval(expr)?;
                    }
                }
            }
        }

        if !option_text.is_empty() {
            self.pending_options.push((option_text, goto_target));
        }

        Ok(Value::Null)
    }

    fn eval_as(&mut self, params: &[(String, Expr)]) -> Result<Value, PslError> {
        // [[as: label]: text]
        // Display narrative text, optionally with a label
        let mut label = String::new();
        let mut text_content = String::new();

        for (key, expr) in params {
            if key == "id" {
                // Skip Item placeholder which indicates empty target
                if expr != &Expr::Item {
                    let val = self.eval(expr)?;
                    if let Value::Text(lbl) = val {
                        label = lbl;
                    } else {
                        // For non-text expressions, try to convert
                        match val {
                            Value::Container(_) => label = "Container".to_string(),
                            _ => {}
                        }
                    }
                }
            } else if key == "body" {
                let val = self.eval(expr)?;
                if let Value::Text(t) = val {
                    text_content = t;
                }
            }
        }

        if !label.is_empty() {
            self.output.push(format!("[AS: {}] {}", label, text_content));
        } else {
            self.output.push(format!("[TEXT] {}", text_content));
        }

        Ok(Value::Null)
    }

    fn eval_goto_target(&mut self, params: &[(String, Expr)]) -> Result<String, PslError> {
        // Parse [goto: [chapter: name]] or [goto: [section: name]]
        // Can be called with either "body" (from double-bracketed) or "target" (from single-bracketed)
        for (key, expr) in params {
            if key == "body" || key == "target" {
                match expr {
                    Expr::Chapter(name) => return Ok(format!("chapter::{}", name)),
                    Expr::Section(name) => return Ok(format!("section::{}", name)),
                    Expr::Keyword { name, params: inner_params } => {
                        // Might be a nested keyword, try to evaluate it
                        if name == "chapter" || name == "section" {
                            return self.eval_goto_target(inner_params);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok("unknown".to_string())
    }

    fn eval_goto(&mut self, params: &[(String, Expr)]) -> Result<Value, PslError> {
        // Navigate to chapter or section
        let target = self.eval_goto_target(params)?;
        
        // Update current location
        if target.starts_with("chapter::") {
            let chapter = target.strip_prefix("chapter::").unwrap_or("").to_string();
            self.current_chapter = Some(chapter);
            self.current_section = None;
        } else if target.starts_with("section::") {
            let section = target.strip_prefix("section::").unwrap_or("").to_string();
            self.current_section = Some(section);
        }
        
        self.output.push(format!("[GOTO] {}", target));
        Ok(Value::Text(target))
    }

    fn eval_chapter(&mut self, params: &[(String, Expr)]) -> Result<Value, PslError> {
        // [[chapter: name]: display_text]
        // Chapter marker - set current chapter and display heading
        let mut chapter_name = String::new();
        let mut display_text = String::new();

        for (key, expr) in params {
            if key == "id" {
                if let Expr::Chapter(name) = expr {
                    chapter_name = name.clone();
                    self.current_chapter = Some(name.clone());
                }
            } else if key == "body" {
                let val = self.eval(expr)?;
                if let Value::Text(text) = val {
                    display_text = text;
                }
            }
        }

        if !display_text.is_empty() {
            self.output.push(format!("[CHAPTER: {}] {}", chapter_name, display_text));
        }

        Ok(Value::Null)
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
