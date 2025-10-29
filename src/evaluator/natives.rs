use std::{
    io,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::evaluator::{
    Evaluator, env::{Env, EnvPtr}, runtime_err::EvalResult, value::{Callable, Value}
};

pub struct Natives;

impl Natives {
    pub fn get_natives() -> EnvPtr {
        let natives = Env::new();

        natives
            .borrow_mut()
            .define("print".into(), Value::Callable(Rc::new(FnPrint)));
        natives
            .borrow_mut()
            .define("read".into(), Value::Callable(Rc::new(FnRead)));
        natives
            .borrow_mut()
            .define("clock".into(), Value::Callable(Rc::new(FnClock)));

        natives
    }
}

/// print(expr): prints an expression to stdout with a newline
#[derive(Debug)]
struct FnPrint;
impl Callable for FnPrint {
    fn name(&self) -> &str {
        "print"
    }

    fn arity(&self) -> usize {
        1
    }

    fn call(&self, _evaluator: &mut Evaluator, args: Vec<Value>) -> EvalResult<Value> {
        println!("{}", args[0]);
        Ok(Value::Null)
    }
}

/// read() -> Str: reads a string from stdin
#[derive(Debug)]
struct FnRead;
impl Callable for FnRead {
    fn name(&self) -> &str {
        "read"
    }

    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _evaluator: &mut Evaluator, _args: Vec<Value>) -> EvalResult<Value> {
        let mut string = String::new();
        io::stdin()
            .read_line(&mut string)
            .expect("Failed to read line");
        Ok(Value::Str(string.trim().to_string()))
    }
}

/// clock() -> Num: returns millis since unix epoch
#[derive(Debug)]
struct FnClock;
impl Callable for FnClock {
    fn name(&self) -> &str {
        "clock"
    }

    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _evaluator: &mut Evaluator, _args: Vec<Value>) -> EvalResult<Value> {
        let start = SystemTime::now();
        let from_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("time should go forward");
        Ok(Value::Num(from_epoch.as_millis() as f64))
    }
}
