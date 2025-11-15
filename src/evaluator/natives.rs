mod macros;
mod math;
mod rand;
mod p5;
mod sys;
mod term;
mod tui;

use std::{
    cell::RefCell,
    io::{self},
    rc::Rc,
};

use crate::{
    evaluator::{
        Evaluator,
        env::{Env, EnvPtr},
        runtime_err::EvalResult,
        value::{Callable, Value},
    },
    native_fn,
};

pub struct Natives;

impl Natives {
    pub fn get_natives() -> EnvPtr {
        let natives = Env::new();

        // global functions
        natives
            .borrow_mut()
            .define("print".into(), Value::Callable(Rc::new(FnPrint)));
        natives
            .borrow_mut()
            .define("println".into(), Value::Callable(Rc::new(FnPrintln)));
        natives
            .borrow_mut()
            .define("read".into(), Value::Callable(Rc::new(FnRead)));

        // global objects
        natives.borrow_mut().define("Sys".into(), sys::native_sys());
        natives
            .borrow_mut()
            .define("Rand".into(), rand::native_rand());
        natives
            .borrow_mut()
            .define("Math".into(), math::native_math());
        natives
            .borrow_mut()
            .define("Term".into(), term::native_term());
        natives.borrow_mut().define("Tui".into(), tui::native_tui());
        natives.borrow_mut().define("P5".into(), p5::native_p5());

        natives
    }
}

// print(expr)
native_fn!(FnPrint, "print", 1, |_evaluator, args, _cursor| {
    print!("{}", args[0]);
    Ok(Value::Null)
});

// println(expr)
native_fn!(FnPrintln, "println", 1, |_evaluator, args, _cursor| {
    println!("{}", args[0]);
    Ok(Value::Null)
});

// read() -> Str
native_fn!(FnRead, "read", 0, |_evaluator, _args, _cursor| {
    let mut string = String::new();
    io::stdin()
        .read_line(&mut string)
        .expect("Failed to read line");
    Ok(Value::Str(Rc::new(RefCell::new(string.trim().to_string()))))
});
