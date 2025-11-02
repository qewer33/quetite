use std::{cell::RefCell, rc::Rc};

use rustc_hash::FxHashMap;

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
    values: FxHashMap<String, Value>,
}

impl Env {
    pub fn new() -> EnvPtr {
        Rc::new(RefCell::new(Self {
            enclosing: None,
            values: FxHashMap::default(),
        }))
    }

    pub fn enclosed(enclosing: EnvPtr) -> EnvPtr {
        Rc::new(RefCell::new(Self {
            enclosing: Some(enclosing),
            values: FxHashMap::default(),
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

    pub fn assign_at(env_ptr: &EnvPtr, name: &str, val: Value, dist: usize) -> EvalResult<()> {
        let ancestor = Self::ancestor(env_ptr.clone(), dist);
        ancestor.borrow_mut().values.insert(name.to_string(), val);
        Ok(())
    }

    pub fn get_at(env_ptr: &EnvPtr, name: &str, dist: usize, cursor: Cursor) -> EvalResult<Value> {
        let ancestor = Self::ancestor(env_ptr.clone(), dist);
        ancestor
            .borrow()
            .values
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeEvent::error(format!("undefined variable '{}'", name), cursor))
    }

    pub fn ancestor(env_ptr: EnvPtr, dist: usize) -> EnvPtr {
        let mut current = env_ptr;
        for _ in 0..dist {
            let next = current.borrow().enclosing.clone().unwrap();
            current = next;
        }
        current
    }
}
