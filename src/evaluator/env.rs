use std::collections::HashMap;

use crate::evaluator::{
    runtime_err::{EvalResult, RuntimeErr},
    value::Value,
};

#[derive(Debug, Clone)]
pub struct Env {
    enclosing: Option<Box<Env>>,
    values: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn with_enclosing(enclosing: Env) -> Self {
        Self {
            enclosing: Some(Box::new(enclosing)),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, val: Value) {
        self.values.insert(name, val);
    }

    pub fn assign(&mut self, name: String, val: Value) -> EvalResult<()> {
        if self.values.contains_key(&name) {
            self.values.insert(name, val);
            return Ok(());
        }

        if let Some(enclosing) = self.enclosing.as_mut() {
            enclosing.assign(name, val)?;
            return Ok(());
        } 

        Err(RuntimeErr::new(format!("Undefined variable {}", name)))
    }

    pub fn get(&mut self, name: String) -> EvalResult<Value> {
        if let Some(val) = self.values.get(&name).cloned() {
            return Ok(val);
        }

        if let Some(enclosing) = self.enclosing.as_mut() {
            return enclosing.get(name);
        }

        Err(RuntimeErr::new(format!("Undefined variable {}", name)))
    }
}
