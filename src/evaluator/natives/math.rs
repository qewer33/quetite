use std::{
    collections::HashMap,
    f64::consts::{E, PI},
    rc::Rc,
};

use ordered_float::OrderedFloat;

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

const TAU: f64 = PI * 2.0;

pub fn native_math() -> Value {
    let mut methods: HashMap<String, Method> = HashMap::new();

    methods.insert(
        "sin".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathSin), false)),
    );
    methods.insert(
        "cos".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathCos), false)),
    );
    methods.insert(
        "tan".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathTan), false)),
    );
    methods.insert(
        "asin".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathAsin), false)),
    );
    methods.insert(
        "acos".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathAcos), false)),
    );
    methods.insert(
        "atan".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathAtan), false)),
    );
    methods.insert(
        "atan2".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathAtan2), false)),
    );
    methods.insert(
        "sqrt".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathSqrt), false)),
    );
    methods.insert(
        "cbrt".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathCbrt), false)),
    );
    methods.insert(
        "exp".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathExp), false)),
    );
    methods.insert(
        "ln".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathLn), false)),
    );
    methods.insert(
        "log10".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathLog10), false)),
    );
    methods.insert(
        "log".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathLog), false)),
    );
    methods.insert(
        "pow".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathPow), false)),
    );
    methods.insert(
        "hypot".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathHypot), false)),
    );
    methods.insert(
        "pi".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathPi), false)),
    );
    methods.insert(
        "tau".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathTau), false)),
    );
    methods.insert(
        "e".into(),
        Method::Native(NativeMethod::new(Rc::new(FnMathE), false)),
    );

    Value::Obj(Rc::new(Object::new("Math".into(), methods)))
}

// sin(x) -> Num
native_fn!(FnMathSin, "sin", 1, |_evaluator, args, cursor| {
    let x = args[0].check_num(cursor, Some("argument".into()))?;
    Ok(Value::Num(OrderedFloat(x.sin())))
});

// cos(x) -> Num
native_fn!(FnMathCos, "cos", 1, |_evaluator, args, cursor| {
    let x = args[0].check_num(cursor, Some("argument".into()))?;
    Ok(Value::Num(OrderedFloat(x.cos())))
});

// tan(x) -> Num
native_fn!(FnMathTan, "tan", 1, |_evaluator, args, cursor| {
    let x = args[0].check_num(cursor, Some("argument".into()))?;
    Ok(Value::Num(OrderedFloat(x.tan())))
});

// asin(x) -> Num
native_fn!(FnMathAsin, "asin", 1, |_evaluator, args, cursor| {
    let x = args[0].check_num(cursor, Some("argument".into()))?;
    Ok(Value::Num(OrderedFloat(x.asin())))
});

// acos(x) -> Num
native_fn!(FnMathAcos, "acos", 1, |_evaluator, args, cursor| {
    let x = args[0].check_num(cursor, Some("argument".into()))?;
    Ok(Value::Num(OrderedFloat(x.acos())))
});

// atan(x) -> Num
native_fn!(FnMathAtan, "atan", 1, |_evaluator, args, cursor| {
    let x = args[0].check_num(cursor, Some("argument".into()))?;
    Ok(Value::Num(OrderedFloat(x.atan())))
});

// atan2(y, x) -> Num
native_fn!(FnMathAtan2, "atan2", 2, |_evaluator, args, cursor| {
    let y = args[0].check_num(cursor, Some("y argument".into()))?;
    let x = args[1].check_num(cursor, Some("x argument".into()))?;
    Ok(Value::Num(OrderedFloat(y.atan2(x))))
});

// sqrt(x) -> Num
native_fn!(FnMathSqrt, "sqrt", 1, |_evaluator, args, cursor| {
    let x = args[0].check_num(cursor, Some("argument".into()))?;
    Ok(Value::Num(OrderedFloat(x.sqrt())))
});

// cbrt(x) -> Num
native_fn!(FnMathCbrt, "cbrt", 1, |_evaluator, args, cursor| {
    let x = args[0].check_num(cursor, Some("argument".into()))?;
    Ok(Value::Num(OrderedFloat(x.cbrt())))
});

// exp(x) -> Num
native_fn!(FnMathExp, "exp", 1, |_evaluator, args, cursor| {
    let x = args[0].check_num(cursor, Some("argument".into()))?;
    Ok(Value::Num(OrderedFloat(x.exp())))
});

// ln(x) -> Num
native_fn!(FnMathLn, "ln", 1, |_evaluator, args, cursor| {
    let x = args[0].check_num(cursor, Some("argument".into()))?;
    if x <= 0.0 {
        return Err(RuntimeEvent::error(
            "Math.ln expects argument > 0".into(),
            cursor,
        ));
    }
    Ok(Value::Num(OrderedFloat(x.ln())))
});

// log10(x) -> Num
native_fn!(FnMathLog10, "log10", 1, |_evaluator, args, cursor| {
    let x = args[0].check_num(cursor, Some("argument".into()))?;
    if x <= 0.0 {
        return Err(RuntimeEvent::error(
            "Math.log10 expects argument > 0".into(),
            cursor,
        ));
    }
    Ok(Value::Num(OrderedFloat(x.log10())))
});

// log(value, base) -> Num
native_fn!(FnMathLog, "log", 2, |_evaluator, args, cursor| {
    let value = args[0].check_num(cursor, Some("value".into()))?;
    let base = args[1].check_num(cursor, Some("base".into()))?;
    if value <= 0.0 {
        return Err(RuntimeEvent::error(
            "Math.log expects value > 0".into(),
            cursor,
        ));
    }
    if base <= 0.0 || (base - 1.0).abs() < f64::EPSILON {
        return Err(RuntimeEvent::error(
            "Math.log expects base > 0 and != 1".into(),
            cursor,
        ));
    }
    Ok(Value::Num(OrderedFloat(value.log(base))))
});

// pow(base, exp) -> Num
native_fn!(FnMathPow, "pow", 2, |_evaluator, args, cursor| {
    let base = args[0].check_num(cursor, Some("base".into()))?;
    let exp = args[1].check_num(cursor, Some("exponent".into()))?;
    Ok(Value::Num(OrderedFloat(base.powf(exp))))
});

// hypot(a, b) -> Num
native_fn!(FnMathHypot, "hypot", 2, |_evaluator, args, cursor| {
    let a = args[0].check_num(cursor, Some("a".into()))?;
    let b = args[1].check_num(cursor, Some("b".into()))?;
    Ok(Value::Num(OrderedFloat(a.hypot(b))))
});

// pi() -> Num
native_fn!(FnMathPi, "pi", 0, |_evaluator, _args, _cursor| {
    Ok(Value::Num(OrderedFloat(PI)))
});

// tau() -> Num
native_fn!(FnMathTau, "tau", 0, |_evaluator, _args, _cursor| {
    Ok(Value::Num(OrderedFloat(TAU)))
});

// e() -> Num
native_fn!(FnMathE, "e", 0, |_evaluator, _args, _cursor| {
    Ok(Value::Num(OrderedFloat(E)))
});
