use core::fmt;
use std::collections::HashMap;

pub type Env = HashMap<String, i64>;

#[derive(Debug, Clone)]
pub enum EvalError {
    UnboundVariable(String),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::UnboundVariable(v) => write!(f, "Undefined variable: '{v}'"),
        }
    }
}
