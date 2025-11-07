use std::{
    cell::RefCell,
    collections::HashMap,
    io,
    rc::Rc,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use ordered_float::OrderedFloat;
use rand::Rng;

use crate::evaluator::{
    Evaluator,
    env::{Env, EnvPtr},
    object::{Method, NativeMethod, Object},
    runtime_err::EvalResult,
    value::{Callable, Value},
};

#[macro_export]
macro_rules! native_fn {
    ($name:ident, $str_name:expr, $arity:expr, |$evaluator:ident, $args:ident| $body:block) => {
        #[derive(Debug)]
        struct $name;
        impl Callable for $name {
            fn name(&self) -> &str {
                $str_name
            }
            fn arity(&self) -> usize {
                $arity
            }
            fn call(&self, $evaluator: &mut Evaluator, $args: Vec<Value>) -> EvalResult<Value> {
                $body
            }
        }
    };
}

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
            .borrow_mut()
            .define("sleep".into(), Value::Callable(Rc::new(FnSleep)));
        natives
            .borrow_mut()
            .define("rand".into(), Value::Callable(Rc::new(FnRand)));

        natives
    }
}

// print(expr): prints an expression to stdin
native_fn!(FnPrint, "print", 1, |_evaluator, args| {
    println!("{}", args[0]);
    Ok(Value::Null)
});

// read() -> Str: reads a string from stdin
native_fn!(FnRead, "read", 0, |_evaluator, _args| {
    let mut string = String::new();
    io::stdin()
        .read_line(&mut string)
        .expect("Failed to read line");
    Ok(Value::Str(Rc::new(RefCell::new(string.trim().to_string()))))
});

// clock() -> Num: returns millis since unix epoch
native_fn!(FnClock, "clock", 0, |_evaluator, _args| {
    let start = SystemTime::now();
    let from_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("time should go forward");
    Ok(Value::Num(OrderedFloat(from_epoch.as_millis() as f64)))
});

// sleep(millis): puts main thread to sleep for given milliseconds
native_fn!(FnSleep, "sleep", 1, |_evaluator, args| {
    if let Value::Num(millis) = args[0] {
        thread::sleep(Duration::from_millis(millis.0 as u64));
    }
    Ok(Value::Null)
});

// rand() -> Num: returns a random number between 0 and 1
native_fn!(FnRand, "rand", 0, |_evaluator, args| {
    let mut rng = rand::rng();
    Ok(Value::Num(OrderedFloat(rng.random())))
});