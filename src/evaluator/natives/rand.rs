use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ordered_float::OrderedFloat;
use rand::Rng;

use crate::{
    evaluator::{
        runtime_err::RuntimeEvent,
        Callable,
        EvalResult,
        Evaluator,
        object::{Method, NativeMethod, Object},
        value::Value,
    },
    native_fn,
};

const RAND_STRING_CHARSET: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

pub fn native_rand() -> Value {
    let mut methods: HashMap<String, Method> = HashMap::new();

    methods.insert(
        "num".into(),
        Method::Native(NativeMethod::new(Rc::new(FnRandNum), false)),
    );
    methods.insert(
        "bool".into(),
        Method::Native(NativeMethod::new(Rc::new(FnRandBool), false)),
    );
    methods.insert(
        "list".into(),
        Method::Native(NativeMethod::new(Rc::new(FnRandList), false)),
    );
    methods.insert(
        "string".into(),
        Method::Native(NativeMethod::new(Rc::new(FnRandString), false)),
    );
    methods.insert(
        "range".into(),
        Method::Native(NativeMethod::new(Rc::new(FnRandRange), false)),
    );
    methods.insert(
        "int".into(),
        Method::Native(NativeMethod::new(Rc::new(FnRandInt), false)),
    );

    Value::Obj(Rc::new(Object::new("Rand".into(), methods)))
}

// rand() -> Num (0..1)
native_fn!(FnRandNum, "num", 0, |_evaluator, _args, _cursor| {
    let mut rng = rand::rng();
    Ok(Value::Num(OrderedFloat(rng.random())))
});

// rand_bool() -> Bool
native_fn!(FnRandBool, "bool", 0, |_evaluator, _args, _cursor| {
    let mut rng = rand::rng();
    Ok(Value::Bool(rng.random()))
});

// rand_list(list: List) -> Value
native_fn!(FnRandList, "list", 1, |_evaluator, args, cursor| {
    let rc_list = args[0].check_list(cursor, Some("list argument".into()))?;
    let list = rc_list.borrow();
    if list.is_empty() {
        return Err(RuntimeEvent::error(
            "cannot choose a random element from an empty list".into(),
            cursor,
        ));
    }
    let mut rng = rand::rng();
    let idx = rng.random_range(0..list.len());
    Ok(list[idx].clone())
});

// rand_string(len: Num) -> Str
native_fn!(FnRandString, "string", 1, |_evaluator, args, cursor| {
    let len_num = args[0].check_num(cursor, Some("string length".into()))?;
    if len_num < 0.0 {
        return Err(RuntimeEvent::error(
            "string length must be non-negative".into(),
            cursor,
        ));
    }
    if (len_num.fract()).abs() > f64::EPSILON {
        return Err(RuntimeEvent::error(
            "string length must be an integer value".into(),
            cursor,
        ));
    }
    let len = len_num as usize;
    let mut rng = rand::rng();
    let result: String = (0..len)
        .map(|_| {
            let idx = rng.random_range(0..RAND_STRING_CHARSET.len());
            RAND_STRING_CHARSET[idx] as char
        })
        .collect();
    Ok(Value::Str(Rc::new(RefCell::new(result))))
});

// rand_range(min: Num, max: Num) -> Num
native_fn!(FnRandRange, "range", 2, |_evaluator, args, cursor| {
    let min = args[0].check_num(cursor, Some("min value".into()))?;
    let max = args[1].check_num(cursor, Some("max value".into()))?;
    if max <= min {
        return Err(RuntimeEvent::error(
            "max must be greater than min when calling Rand.range".into(),
            cursor,
        ));
    }
    let mut rng = rand::rng();
    let value = rng.random_range(min..max);
    Ok(Value::Num(OrderedFloat(value)))
});

// rand_int(min: Num, max: Num) -> Num (integer)
native_fn!(FnRandInt, "int", 2, |_evaluator, args, cursor| {
    let min_raw = args[0].check_num(cursor, Some("min value".into()))?;
    let max_raw = args[1].check_num(cursor, Some("max value".into()))?;
    if (min_raw.fract()).abs() > f64::EPSILON || (max_raw.fract()).abs() > f64::EPSILON {
        return Err(RuntimeEvent::error(
            "Rand.int expects integer bounds".into(),
            cursor,
        ));
    }
    let min = min_raw as i64;
    let max = max_raw as i64;
    if max < min {
        return Err(RuntimeEvent::error(
            "max must be greater than or equal to min when calling Rand.int".into(),
            cursor,
        ));
    }
    let mut rng = rand::rng();
    let value = if max == min {
        min
    } else {
        rng.random_range(min..=max)
    };
    Ok(Value::Num(OrderedFloat(value as f64)))
});
