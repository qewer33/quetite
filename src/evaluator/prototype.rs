use ordered_float::OrderedFloat;

use crate::{evaluator::runtime_err::RuntimeErr, native_fn};
use colored::Colorize;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    evaluator::{
        EvalResult, Evaluator,
        runtime_err::RuntimeEvent,
        value::{Callable, Value},
    },
    lexer::cursor::Cursor,
};

#[macro_export]
macro_rules! proto_method {
    (
        $proto:ident,
        $name:ident,
        $str_name:expr,
        $arity:expr,
        |$evaluator:ident, $args:ident, $cursor:ident, $recv:ident| $body:block
    ) => {
        native_fn!($name, $str_name, $arity, |$evaluator, $args, $cursor| {
            // receiver is always arg0
            let $recv = $args.get(0).ok_or_else(|| {
                RuntimeEvent::error(
                    concat!($str_name, " called without receiver").into(),
                    Cursor::new(),
                )
            })?;

            $body
        });

        $proto.add_method($str_name.to_string(), std::rc::Rc::new($name));
    };
}

// Macro for color/style methods
macro_rules! str_color_method {
    ($proto:ident, $name:ident, $method_name:expr, $colorize:ident) => {
        proto_method!(
            $proto,
            $name,
            $method_name,
            0,
            |_evaluator, args, _cursor, recv| {
                if let Value::Str(s) = recv {
                    Ok(Value::Str(Rc::new(RefCell::new(
                        s.borrow().$colorize().to_string(),
                    ))))
                } else {
                    Ok(recv.clone())
                }
            }
        );
    };
}

pub struct Prototype {
    pub name: String,
    methods: HashMap<String, Rc<dyn Callable>>,
    parent: Option<Rc<Prototype>>,
}

impl Prototype {
    pub fn new(name: String) -> Self {
        Self {
            name,
            methods: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(name: String, parent: &Rc<Prototype>) -> Self {
        Self {
            name,
            methods: HashMap::new(),
            parent: Some(Rc::clone(parent)),
        }
    }

    pub fn add_method(&mut self, name: String, method: Rc<dyn Callable>) {
        self.methods.insert(name, method);
    }

    pub fn get_method(&self, name: String) -> Option<Rc<dyn Callable>> {
        let method = self.methods.get(&name).cloned();
        if let None = method {
            if let Some(parent) = &self.parent {
                return parent.get_method(name);
            }
        }
        method
    }
}

pub struct ValuePrototypes {
    pub list: Prototype,
    pub str: Prototype,
    pub num: Prototype,
    pub bool: Prototype,
}

impl ValuePrototypes {
    pub fn new() -> Self {
        let value = Rc::new(ValuePrototypes::value_proto());
        let list = ValuePrototypes::list_proto(&value);
        let str = ValuePrototypes::str_proto(&value);
        let num = ValuePrototypes::num_proto(&value);
        let bool = ValuePrototypes::bool_proto(&value);
        Self {
            list,
            str,
            num,
            bool,
        }
    }

    pub fn value_proto() -> Prototype {
        let mut proto = Prototype::new("Value".to_string());

        // type() -> Str: returns type of the value
        proto_method!(
            proto,
            ValueType,
            "type",
            0,
            |_evaluator, args, _cursor, recv| {
                Ok(Value::Str(Rc::new(RefCell::new(recv.get_type()))))
            }
        );

        // type_of(type) -> bool: returns true if type of the value matches type, false otherwise
        proto_method!(
            proto,
            ValueTypeOf,
            "type_of",
            1,
            |_evaluator, args, _cursor, recv| {
                if let Value::Str(str) = &args[1] {
                    return Ok(Value::Bool(
                        recv.get_type().to_uppercase() == str.borrow().clone().to_uppercase(),
                    ));
                }
                Ok(Value::Null)
            }
        );

        // type_check(type) -> bool: returns true if type of the value matches type, false otherwise
        proto_method!(
            proto,
            ValueTypeCheck,
            "type_check",
            1,
            |_evaluator, args, cursor, recv| {
                if let Value::Str(str) = &args[1] {
                    return recv.check_type(str.borrow().clone(), cursor).map(|v| Value::Bool(v));
                }
                Ok(Value::Null)
            }
        );

        proto
    }

    pub fn list_proto(value_proto: &Rc<Prototype>) -> Prototype {
        let mut proto = Prototype::with_parent("List".to_string(), value_proto);

        // len() -> Num: returns number of elements
        proto_method!(
            proto,
            ListLen,
            "len",
            0,
            |_evaluator, args, _cursor, recv| {
                if let Value::List(list) = recv {
                    let len = list.borrow().len() as f64;
                    return Ok(Value::Num(len.into()));
                }
                unreachable!()
            }
        );

        // push(value): appends, returns null
        proto_method!(
            proto,
            ListPush,
            "push",
            1,
            |_evaluator, args, _cursor, recv| {
                if let Value::List(list) = recv {
                    let val = args.get(1).cloned().unwrap();
                    list.borrow_mut().push(val);
                    return Ok(Value::Null);
                }
                unreachable!()
            }
        );

        // pop(): removes last, returns it or null
        proto_method!(
            proto,
            ListPop,
            "pop",
            0,
            |_evaluator, args, _cursor, recv| {
                if let Value::List(list) = recv {
                    let mut vec = list.borrow_mut();
                    if let Some(v) = vec.pop() {
                        return Ok(v);
                    } else {
                        return Ok(Value::Null);
                    }
                }
                unreachable!()
            }
        );

        // insert(index, value): inserts value at index
        proto_method!(
            proto,
            ListInsert,
            "insert",
            2,
            |_evaluator, args, _cursor, recv| {
                if let Value::List(list) = recv {
                    if let Value::Num(n) = args[1] {
                        list.borrow_mut().insert(n.0 as usize, args[2].clone());
                    }
                    return Ok(Value::Null);
                }
                unreachable!()
            }
        );

        // remove(index): removes the element at index
        proto_method!(
            proto,
            ListRemove,
            "remove",
            1,
            |_evaluator, args, _cursor, recv| {
                if let Value::List(list) = recv {
                    if let Value::Num(n) = args[1] {
                        list.borrow_mut().remove(n.0 as usize);
                    }
                    return Ok(Value::Null);
                }
                unreachable!()
            }
        );

        // last(): returns the last element
        proto_method!(
            proto,
            ListLast,
            "last",
            0,
            |_evaluator, args, _cursor, recv| {
                if let Value::List(list) = recv {
                    if let Some(val) = list.borrow().last() {
                        return Ok(val.clone());
                    }
                    return Ok(Value::Null);
                }
                unreachable!()
            }
        );

        // first(): returns the first element
        proto_method!(
            proto,
            ListFirst,
            "first",
            0,
            |_evaluator, args, _cursor, recv| {
                if let Value::List(list) = recv {
                    if let Some(val) = list.borrow().first() {
                        return Ok(val.clone());
                    }
                    return Ok(Value::Null);
                }
                unreachable!()
            }
        );

        // contains(vals): returns true if list contains value, false otherwise
        proto_method!(
            proto,
            ListContains,
            "contains",
            1,
            |_evaluator, args, _cursor, recv| {
                if let Value::List(list) = recv {
                    return Ok(Value::Bool(list.borrow().contains(&args[1])));
                }
                unreachable!()
            }
        );

        proto
    }

    pub fn str_proto(value_proto: &Rc<Prototype>) -> Prototype {
        let mut proto = Prototype::with_parent("Str".to_string(), value_proto);

        // parse_num() -> Num: parses the Str to a Num
        proto_method!(
            proto,
            StrParseNum,
            "parse_num",
            0,
            |_evaluator, _cursor, args, recv| {
                if let Value::Str(str) = recv {
                    if let Ok(num) = str.borrow().parse::<f64>() {
                        return Ok(Value::Num(OrderedFloat(num)));
                    } else {
                        return Ok(Value::Null);
                    }
                }
                unreachable!()
            }
        );

        // len() -> Str: returns the length of the string
        proto_method!(
            proto,
            StrLen,
            "len",
            0,
            |_evaluator, args, _cursor, recv| {
                if let Value::Str(str) = recv {
                    return Ok(Value::Num(OrderedFloat(str.borrow().len() as f64)));
                }
                unreachable!()
            }
        );

        // Foreground colors
        str_color_method!(proto, StrBlack, "black", black);
        str_color_method!(proto, StrRed, "red", red);
        str_color_method!(proto, StrGreen, "green", green);
        str_color_method!(proto, StrYellow, "yellow", yellow);
        str_color_method!(proto, StrBlue, "blue", blue);
        str_color_method!(proto, StrMagenta, "magenta", magenta);
        str_color_method!(proto, StrCyan, "cyan", cyan);
        str_color_method!(proto, StrWhite, "white", white);

        // Bright colors
        str_color_method!(proto, StrBrightBlack, "bright_black", bright_black);
        str_color_method!(proto, StrBrightRed, "bright_red", bright_red);
        str_color_method!(proto, StrBrightGreen, "bright_green", bright_green);
        str_color_method!(proto, StrBrightYellow, "bright_yellow", bright_yellow);
        str_color_method!(proto, StrBrightBlue, "bright_blue", bright_blue);
        str_color_method!(proto, StrBrightMagenta, "bright_magenta", bright_magenta);
        str_color_method!(proto, StrBrightCyan, "bright_cyan", bright_cyan);
        str_color_method!(proto, StrBrightWhite, "bright_white", bright_white);

        // Styles
        str_color_method!(proto, StrBold, "bold", bold);
        str_color_method!(proto, StrDim, "dim", dimmed);
        str_color_method!(proto, StrItalic, "italic", italic);
        str_color_method!(proto, StrUnderline, "underline", underline);
        str_color_method!(proto, StrBlink, "blink", blink);
        str_color_method!(proto, StrReverse, "reverse", reversed);
        str_color_method!(proto, StrStrikethrough, "strikethrough", strikethrough);

        // Background colors
        str_color_method!(proto, StrOnBlack, "on_black", on_black);
        str_color_method!(proto, StrOnRed, "on_red", on_red);
        str_color_method!(proto, StrOnGreen, "on_green", on_green);
        str_color_method!(proto, StrOnYellow, "on_yellow", on_yellow);
        str_color_method!(proto, StrOnBlue, "on_blue", on_blue);
        str_color_method!(proto, StrOnMagenta, "on_magenta", on_magenta);
        str_color_method!(proto, StrOnCyan, "on_cyan", on_cyan);
        str_color_method!(proto, StrOnWhite, "on_white", on_white);

        proto
    }

    pub fn num_proto(value_proto: &Rc<Prototype>) -> Prototype {
        let mut proto = Prototype::with_parent("Num".to_string(), value_proto);

        // abs() -> Num: returns absolute value of number
        proto_method!(
            proto,
            NumAbs,
            "abs",
            0,
            |_evaluator, args, _cursor, recv| {
                if let Value::Num(num) = recv {
                    return Ok(Value::Num(OrderedFloat(num.abs())));
                }
                unreachable!()
            }
        );

        // round() -> Num: returns the number rounded to the nearest integer
        proto_method!(
            proto,
            NumRound,
            "round",
            0,
            |_evaluator, args, _cursor, recv| {
                if let Value::Num(num) = recv {
                    return Ok(Value::Num(OrderedFloat(num.round())));
                }
                unreachable!()
            }
        );

        // ceil() -> Num: returns the number rounded to the smallest larger integer
        proto_method!(
            proto,
            NumCeil,
            "ceil",
            0,
            |_evaluator, args, _cursor, recv| {
                if let Value::Num(num) = recv {
                    return Ok(Value::Num(OrderedFloat(num.ceil())));
                }
                unreachable!()
            }
        );

        // floor() -> Num: returns the number rounded to the largest smaller integer
        proto_method!(
            proto,
            NumFloor,
            "floor",
            0,
            |_evaluator, args, _cursor, recv| {
                if let Value::Num(num) = recv {
                    return Ok(Value::Num(OrderedFloat(num.floor())));
                }
                unreachable!()
            }
        );

        // clamp(min, max) -> Num: returns the number clamped between min and max
        proto_method!(
            proto,
            NumClamp,
            "clamp",
            2,
            |_evaluator, args, _cursor, recv| {
                if let Value::Num(num) = recv {
                    let min = if let Value::Num(n) = args[1] {
                        n.0
                    } else {
                        return Ok(Value::Null);
                    };
                    let max = if let Value::Num(n) = args[2] {
                        n.0
                    } else {
                        return Ok(Value::Null);
                    };

                    return Ok(Value::Num(OrderedFloat(num.0.clamp(min, max))));
                }
                unreachable!()
            }
        );

        // to_str() -> Num: returns the number as an Str
        proto_method!(
            proto,
            NumToStr,
            "to_str",
            0,
            |_evaluator, args, _cursor, recv| {
                if let Value::Num(num) = recv {
                    return Ok(Value::Str(Rc::new(RefCell::new(num.to_string()))));
                }
                unreachable!()
            }
        );

        proto
    }

    pub fn bool_proto(value_proto: &Rc<Prototype>) -> Prototype {
        let mut proto = Prototype::with_parent("Bool".to_string(), value_proto);

        // to_num() -> Num: returns 1 if true, 0 if false
        proto_method!(
            proto,
            BoolToNum,
            "to_num",
            0,
            |_evaluator, args, _cursor, recv| {
                if let Value::Bool(b) = recv {
                    if *b {
                        return Ok(Value::Num(OrderedFloat(1.0)));
                    } else {
                        return Ok(Value::Num(OrderedFloat(0.0)));
                    }
                }
                unreachable!()
            }
        );

        proto
    }
}

#[derive(Debug)]
pub struct BoundMethod {
    pub receiver: Value,
    pub method: Rc<dyn Callable>,
}

impl Callable for BoundMethod {
    fn name(&self) -> &str {
        self.method.name()
    }

    fn arity(&self) -> usize {
        // method will receive `self` as arg[0], so from the caller’s POV arity stays the same
        self.method.arity()
    }

    fn call(
        &self,
        evaluator: &mut Evaluator,
        mut args: Vec<Value>,
        cursor: Cursor,
    ) -> EvalResult<Value> {
        // prepend the receiver so native methods can do: let this = &args[0];
        let mut real_args = Vec::with_capacity(args.len() + 1);
        real_args.push(self.receiver.clone());
        real_args.append(&mut args);
        self.method.call(evaluator, real_args, cursor)
    }
}
