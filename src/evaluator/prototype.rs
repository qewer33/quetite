use ordered_float::OrderedFloat;

use crate::native_fn;
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
        |$evaluator:ident, $args:ident, $recv:ident| $body:block
    ) => {
        native_fn!($name, $str_name, $arity, |$evaluator, $args| {
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

pub struct Prototype {
    pub name: String,
    methods: HashMap<String, Rc<dyn Callable>>,
}

impl Prototype {
    pub fn new(name: String) -> Self {
        Self {
            name,
            methods: HashMap::new(),
        }
    }

    pub fn add_method(&mut self, name: String, method: Rc<dyn Callable>) {
        self.methods.insert(name, method);
    }

    pub fn get_method(&self, name: String) -> Option<Rc<dyn Callable>> {
        self.methods.get(&name).cloned()
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
        let list = ValuePrototypes::list_proto();
        let str = ValuePrototypes::str_proto();
        let num = ValuePrototypes::num_proto();
        let bool = ValuePrototypes::bool_proto();
        Self {
            list,
            str,
            num,
            bool,
        }
    }

    pub fn list_proto() -> Prototype {
        let mut proto = Prototype::new("List".to_string());

        // type() -> Str: returns type of the value
        proto_method!(proto, ListType, "type", 0, |_evaluator, args, recv| {
            Ok(Value::Str(Rc::new(RefCell::new(recv.get_type()))))
        });

        // len() -> Num: returns number of elements
        proto_method!(proto, ListLen, "len", 0, |_evaluator, args, recv| {
            if let Value::List(list) = recv {
                let len = list.borrow().len() as f64;
                return Ok(Value::Num(len.into()));
            }
            unreachable!()
        });

        // push(value): appends, returns null
        proto_method!(proto, ListPush, "push", 1, |_evaluator, args, recv| {
            if let Value::List(list) = recv {
                let val = args.get(1).cloned().unwrap();
                list.borrow_mut().push(val);
                return Ok(Value::Null);
            }
            unreachable!()
        });

        // pop(): removes last, returns it or null
        proto_method!(proto, ListPop, "pop", 0, |_evaluator, args, recv| {
            if let Value::List(list) = recv {
                let mut vec = list.borrow_mut();
                if let Some(v) = vec.pop() {
                    return Ok(v);
                } else {
                    return Ok(Value::Null);
                }
            }
            unreachable!()
        });

        proto
    }

    pub fn str_proto() -> Prototype {
        let mut proto = Prototype::new("Str".to_string());

        // type() -> Str: returns type of the value
        proto_method!(proto, StrType, "type", 0, |_evaluator, args, recv| {
            Ok(Value::Str(Rc::new(RefCell::new(recv.get_type()))))
        });

        // parse_num() -> Num: parses the Str to a Num
        proto_method!(
            proto,
            StrParseNum,
            "parse_num",
            0,
            |_evaluator, args, recv| {
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

        proto
    }

    pub fn num_proto() -> Prototype {
        let mut proto = Prototype::new("Num".to_string());

        // type() -> Str: returns type of the value
        proto_method!(proto, NumType, "type", 0, |_evaluator, args, recv| {
            Ok(Value::Str(Rc::new(RefCell::new(recv.get_type()))))
        });

        // round() -> Num: returns the number rounded to the nearest integer
        proto_method!(proto, NumRound, "round", 0, |_evaluator, args, recv| {
            if let Value::Num(num) = recv {
                return Ok(Value::Num(OrderedFloat(num.round())));
            }
            unreachable!()
        });

        proto
    }

    pub fn bool_proto() -> Prototype {
        let mut proto = Prototype::new("Bool".to_string());

        // type() -> Str: returns type of the value
        proto_method!(proto, BoolType, "type", 0, |_evaluator, args, recv| {
            Ok(Value::Str(Rc::new(RefCell::new(recv.get_type()))))
        });

        // as_num() -> Num: returns 1 if true, 0 if false
        proto_method!(proto, BoolAsNum, "as_num", 0, |_evaluator, args, recv| {
            if let Value::Bool(b) = recv {
                if *b {
                    return Ok(Value::Num(OrderedFloat(1.0)));
                } else {
                    return Ok(Value::Num(OrderedFloat(0.0)));
                }
            }
            unreachable!()
        });

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

    fn call(&self, evaluator: &mut Evaluator, mut args: Vec<Value>) -> EvalResult<Value> {
        // prepend the receiver so native methods can do: let this = &args[0];
        let mut real_args = Vec::with_capacity(args.len() + 1);
        real_args.push(self.receiver.clone());
        real_args.append(&mut args);
        self.method.call(evaluator, real_args)
    }
}
