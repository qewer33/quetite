use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    ops::Deref,
    rc::Rc,
};

use ordered_float::OrderedFloat;

use crate::{
    evaluator::{
        Evaluator,
        object::{Instance, Object},
        prototype::{Prototype, ValuePrototypes},
        runtime_err::{ErrKind, EvalResult, RuntimeErr, RuntimeEvent},
    },
    lexer::cursor::Cursor,
};

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Num(OrderedFloat<f64>),
    Str(Rc<RefCell<String>>),
    List(Rc<RefCell<Vec<Value>>>),
    Dict(Rc<RefCell<HashMap<ValueKey, Value>>>),
    Callable(Rc<dyn Callable>),
    Obj(Rc<Object>),
    ObjInstance(Rc<RefCell<Instance>>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Num(n) => write!(f, "{}", n.0),
            Value::Str(s) => write!(f, "{}", s.borrow()),
            Value::List(l) => {
                write!(
                    f,
                    "[{}]",
                    l.borrow()
                        .iter()
                        .map(|e| if e.get_type() == "Str" {
                            format!("\"{}\"", e)
                        } else {
                            e.to_string()
                        })
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Value::Dict(d) => {
                let entries = d
                    .borrow()
                    .iter()
                    .map(|(key, value)| {
                        let key_str = match key {
                            ValueKey::Str(s) => format!("\"{}\"", s),
                            ValueKey::Bool(b) => b.to_string(),
                            ValueKey::Num(n) => n.0.to_string(),
                            ValueKey::Null => "Null".into(),
                        };
                        let val_str = if value.get_type() == "Str" {
                            format!("\"{}\"", value)
                        } else {
                            value.to_string()
                        };
                        format!("  {}: {}", key_str, val_str)
                    })
                    .collect::<Vec<String>>()
                    .join(",\n");

                if entries.is_empty() {
                    write!(f, "{{}}")
                } else {
                    write!(f, "{{\n{}\n}}", entries)
                }
            }
            Value::Callable(c) => write!(f, "{:?}", c),
            Value::Obj(o) => write!(f, "{}", o.name),
            Value::ObjInstance(i) => write!(f, "{}", i.borrow().to_string()),
        }
    }
}

impl Value {
    pub fn prototype<'a>(&self, prototypes: &'a ValuePrototypes) -> Option<&'a Prototype> {
        match self {
            Value::Num(_) => Some(&prototypes.num),
            Value::Str(_) => Some(&prototypes.str),
            Value::List(_) => Some(&prototypes.list),
            Value::Bool(_) => Some(&prototypes.bool),
            Value::Dict(_) => Some(&prototypes.dict),
            _ => None,
        }
    }

    pub fn get_type(&self) -> String {
        match self {
            Value::Null => "Null".to_string(),
            Value::Bool(_) => "Bool".to_string(),
            Value::Num(_) => "Num".to_string(),
            Value::Str(_) => "Str".to_string(),
            Value::List(_) => "List".to_string(),
            Value::Dict(_) => "Dict".to_string(),
            Value::Callable(_) => "Fn".to_string(),
            Value::Obj(_) => "Obj".to_string(),
            Value::ObjInstance(inst) => inst.borrow().obj.name.clone(),
        }
    }

    pub fn check_type(&self, expected: String, cursor: Cursor) -> EvalResult<bool> {
        if expected.to_uppercase() == self.get_type().to_uppercase() {
            return Ok(true);
        }
        Err(RuntimeEvent::Err(RuntimeErr::new(
            ErrKind::Type,
            format!(
                "expected value of type {}, found {}",
                expected,
                self.get_type()
            ),
            cursor,
        )))
    }

    pub fn check_num(&self, cursor: Cursor, name: Option<String>) -> EvalResult<f64> {
        if let Value::Num(f) = self {
            return Ok(f.0);
        }
        let val = match name {
            Some(val) => val,
            None => "value".to_string(),
        };
        Err(RuntimeEvent::Err(RuntimeErr::new(
            ErrKind::Type,
            format!("expected {} of type Num, found {}", val, self.get_type()),
            cursor,
        )))
    }

    pub fn check_str(
        &self,
        cursor: Cursor,
        name: Option<String>,
    ) -> EvalResult<Rc<RefCell<String>>> {
        if let Value::Str(str) = self {
            return Ok(Rc::clone(&str));
        }
        let val = match name {
            Some(val) => val,
            None => "value".to_string(),
        };
        Err(RuntimeEvent::Err(RuntimeErr::new(
            ErrKind::Type,
            format!("expected {} of type Str, found {}", val, self.get_type()),
            cursor,
        )))
    }

    pub fn check_bool(&self, cursor: Cursor, name: Option<String>) -> EvalResult<bool> {
        if let Value::Bool(val) = self {
            return Ok(*val);
        }
        let val = match name {
            Some(val) => val,
            None => "value".to_string(),
        };
        Err(RuntimeEvent::Err(RuntimeErr::new(
            ErrKind::Type,
            format!("expected {} of type Bool, found {}", val, self.get_type()),
            cursor,
        )))
    }

    pub fn check_list(
        &self,
        cursor: Cursor,
        name: Option<String>,
    ) -> EvalResult<Rc<RefCell<Vec<Value>>>> {
        if let Value::List(list) = self {
            return Ok(Rc::clone(&list));
        }
        let val = match name {
            Some(val) => val,
            None => "value".to_string(),
        };
        Err(RuntimeEvent::Err(RuntimeErr::new(
            ErrKind::Type,
            format!("expected {} of type List, found {}", val, self.get_type()),
            cursor,
        )))
    }

    pub fn is_equal(&self, other: &Value) -> bool {
        match self {
            Value::Null => {
                if let Value::Null = other {
                    return true;
                }
                return false;
            }
            Value::Bool(b) => {
                if let Value::Bool(ob) = other {
                    return b == ob;
                }
                return false;
            }
            Value::Num(n) => {
                if let Value::Num(on) = other {
                    return n == on;
                }
                return false;
            }
            Value::Str(s) => {
                if let Value::Str(os) = other {
                    return s == os;
                }
                return false;
            }
            Value::List(_) => {
                // TODO: implement list eq
                return false;
            }
            Value::Dict(_) => {
                // TODO: implement dict eq
                return false;
            }
            Value::Obj(o) => {
                if let Value::Obj(oo) = other {
                    return o.name == oo.name;
                }
                return false;
            }
            Value::Callable(c) => {
                if let Value::Callable(oc) = other {
                    return c.name() == oc.name();
                }
                return false;
            }
            Value::ObjInstance(_) => {
                // TODO: implement obj instance eq
                return false;
            }
        }
    }

    pub fn is_truthy(&self) -> bool {
        // false, 0 and Null are falsey values, everything else is thruthy
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Num(n) => *n != 0.,
            _ => true,
        }
    }

    pub fn add_assign(&self, rhs: Value, cursor: Cursor) -> EvalResult<Value> {
        match self {
            // number += number
            Value::Num(n) => {
                if let Value::Num(m) = rhs {
                    Ok(Value::Num(OrderedFloat(n.0 + m.0)))
                } else {
                    Err(RuntimeEvent::error(
                        ErrKind::Type,
                        "cannot add-asssign non-Num to Num".into(),
                        cursor,
                    ))
                }
            }

            // string += anything -> string append
            Value::Str(s) => {
                let mut s_mut = s.borrow_mut();
                s_mut.push_str(rhs.to_string().as_str());
                // return same string value (Rc)
                Ok(Value::Str(s.clone()))
            }

            // list += elem -> push
            Value::List(vec) => {
                vec.borrow_mut().push(rhs);
                Ok(Value::List(vec.clone()))
            }

            // you might want to support ObjInstance here, etc.
            _ => Err(RuntimeEvent::error(
                ErrKind::Type,
                "invalid left-hand side for '+='".into(),
                cursor,
            )),
        }
    }

    /// v -= rhs
    pub fn sub_assign(&self, rhs: Value, cursor: Cursor) -> EvalResult<Value> {
        match self {
            Value::Num(n) => {
                if let Value::Num(m) = rhs {
                    Ok(Value::Num(OrderedFloat(n.0 - m.0)))
                } else {
                    Err(RuntimeEvent::error(
                        ErrKind::Type,
                        "cannot sub-assign non-Num from Num".into(),
                        cursor,
                    ))
                }
            }

            // TODO: list -= ???
            // TODO: string -= ???
            _ => Err(RuntimeEvent::error(
                ErrKind::Type,
                "invalid left-hand side for '-='".into(),
                cursor,
            )),
        }
    }
}

pub trait Callable: Debug {
    fn name(&self) -> &str;
    fn arity(&self) -> usize;
    fn call(
        &self,
        evaluator: &mut Evaluator,
        args: Vec<Value>,
        cursor: Cursor,
    ) -> EvalResult<Value>;
}

// Hashable value types that can be used as Dict keys
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ValueKey {
    Null,
    Bool(bool),
    Num(OrderedFloat<f64>),
    Str(String),
}

impl TryFrom<&Value> for ValueKey {
    type Error = ();

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Null => Ok(ValueKey::Null),
            Value::Bool(b) => Ok(ValueKey::Bool(*b)),
            Value::Num(n) => Ok(ValueKey::Num(*n)),
            Value::Str(s) => Ok(ValueKey::Str((*s.deref().borrow().deref()).clone())),
            _ => Err(()),
        }
    }
}

impl Into<Value> for ValueKey {
    fn into(self) -> Value {
        match self {
            ValueKey::Null => Value::Null,
            ValueKey::Bool(b) => Value::Bool(b),
            ValueKey::Num(n) => Value::Num(n),
            ValueKey::Str(s) => Value::Str(Rc::new(RefCell::new(s))),
        }
    }
}
