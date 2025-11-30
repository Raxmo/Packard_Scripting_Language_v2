use std::collections::HashMap;
use std::fmt;

/// Represents a value in PSL
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Text(String),
    Number(f64),
    Flag(bool),
    Item,
    Container(HashMap<String, Value>),
    Null,
}

impl Value {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Value::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_flag(&self) -> Option<bool> {
        match self {
            Value::Flag(b) => Some(*b),
            _ => None,
        }
    }

    pub fn is_item(&self) -> bool {
        matches!(self, Value::Item)
    }

    pub fn as_container(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Container(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_container_mut(&mut self) -> Option<&mut HashMap<String, Value>> {
        match self {
            Value::Container(map) => Some(map),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Text(_) => "text",
            Value::Number(_) => "number",
            Value::Flag(_) => "flag",
            Value::Item => "item",
            Value::Container(_) => "container",
            Value::Null => "null",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Text(s) => !s.is_empty(),
            Value::Number(n) => *n != 0.0,
            Value::Flag(b) => *b,
            Value::Item => true,
            Value::Container(m) => !m.is_empty(),
            Value::Null => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Text(s) => write!(f, "{}", s),
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Flag(b) => write!(f, "{}", if *b { "on" } else { "off" }),
            Value::Item => write!(f, "[item]"),
            Value::Container(_) => write!(f, "[container]"),
            Value::Null => write!(f, "null"),
        }
    }
}
