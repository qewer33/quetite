use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    evaluator::{
        runtime_err::{EvalResult, RuntimeEvent},
        value::Value,
    },
    lexer::cursor::Cursor,
};

pub type EnvPtr = Rc<RefCell<Env>>;

#[derive(Debug)]
pub struct Env {
    enclosing: Option<EnvPtr>,
    values: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> EnvPtr {
        Rc::new(RefCell::new(Self {
            enclosing: None,
            values: HashMap::new(),
        }))
    }

    pub fn enclosed(enclosing: EnvPtr) -> EnvPtr {
        Rc::new(RefCell::new(Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }))
    }

    pub fn define(&mut self, name: String, val: Value) {
        self.values.insert(name, val);
    }

    pub fn assign(&mut self, name: &str, val: Value, cursor: Cursor) -> EvalResult<()> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), val);
            return Ok(());
        }
        if let Some(ref parent) = self.enclosing {
            return parent.borrow_mut().assign(name, val, cursor);
        }
        Err(RuntimeEvent::error(
            format!("undefined variable '{}'", name),
            cursor,
        ))
    }

    pub fn get(&self, name: &str, cursor: Cursor) -> EvalResult<Value> {
        if let Some(val) = self.values.get(name) {
            return Ok(val.clone());
        }
        if let Some(ref parent) = self.enclosing {
            return parent.borrow().get(name, cursor);
        }
        Err(RuntimeEvent::error(
            format!("undefined variable '{}'", name),
            cursor,
        ))
    }
}
