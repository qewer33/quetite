use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use ordered_float::OrderedFloat;

use crate::{
    evaluator::{
        object::{Method, NativeMethod, Object},
        runtime_err::RuntimeEvent,
        Callable,
        EvalResult,
        Evaluator,
        value::Value,
    },
    native_fn,
};

pub fn native_sys() -> Value {
    let mut methods: HashMap<String, Method> = HashMap::new();

    methods.insert(
        "clock".into(),
        Method::Native(NativeMethod::new(Rc::new(FnSysClock), false)),
    );
    methods.insert(
        "sleep".into(),
        Method::Native(NativeMethod::new(Rc::new(FnSysSleep), false)),
    );
    methods.insert(
        "env".into(),
        Method::Native(NativeMethod::new(Rc::new(FnSysEnv), false)),
    );
    methods.insert(
        "args".into(),
        Method::Native(NativeMethod::new(Rc::new(FnSysArgs), false)),
    );
    methods.insert(
        "cwd".into(),
        Method::Native(NativeMethod::new(Rc::new(FnSysCwd), false)),
    );

    Value::Obj(Rc::new(Object::new("Sys".into(), methods)))
}

native_fn!(FnSysClock, "sys_clock", 0, |_evaluator, _args, _cursor| {
    let start = SystemTime::now();
    let from_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("time should go forward");
    Ok(Value::Num(OrderedFloat(from_epoch.as_millis() as f64)))
});

// sleep(ms: Num)
native_fn!(FnSysSleep, "sys_sleep", 1, |_evaluator, args, _cursor| {
    if let Value::Num(millis) = args[0] {
        thread::sleep(Duration::from_millis(millis.0 as u64));
    }
    Ok(Value::Null)
});

// env(name: Str) -> Str | Null
native_fn!(FnSysEnv, "sys_env", 1, |_evaluator, args, cursor| {
    let name_rc = args[0].check_str(cursor, Some("environment variable name".into()))?;
    let key = name_rc.borrow().clone();
    match std::env::var(&key) {
        Ok(val) => Ok(Value::Str(Rc::new(RefCell::new(val)))),
        Err(_) => Ok(Value::Null),
    }
});

// args() -> List<Str>
native_fn!(FnSysArgs, "sys_args", 0, |_evaluator, _args, _cursor| {
    let values = std::env::args()
        .map(|arg| Value::Str(Rc::new(RefCell::new(arg))))
        .collect::<Vec<Value>>();
    Ok(Value::List(Rc::new(RefCell::new(values))))
});

// cwd() -> Str
native_fn!(FnSysCwd, "sys_cwd", 0, |_evaluator, _args, cursor| {
    let cwd = std::env::current_dir().map_err(|err| {
        RuntimeEvent::error(format!("failed to read current directory: {err}"), cursor)
    })?;
    Ok(Value::Str(Rc::new(RefCell::new(
        cwd.to_string_lossy().to_string(),
    ))))
});
