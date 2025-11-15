use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    evaluator::{
        env::Env,
        function::Function,
        runtime_err::{EvalResult, RuntimeEvent},
        value::{Callable, Value},
    },
    lexer::cursor::Cursor,
    parser::stmt::StmtKind,
};

#[derive(Debug, Clone)]
pub enum Method {
    User(Function),
    Native(NativeMethod),
}

#[derive(Debug, Clone)]
pub struct NativeMethod {
    pub callable: Rc<dyn Callable>,
    pub bound: bool,
    pub bind: Option<Value>,
}

impl NativeMethod {
    pub fn new(callable: Rc<dyn Callable>, bound: bool) -> Self {
        Self {
            callable,
            bound,
            bind: None,
        }
    }
}

impl Callable for NativeMethod {
    fn name(&self) -> &str {
        self.callable.name()
    }

    fn arity(&self) -> usize {
        self.callable.arity()
    }

    fn call(
        &self,
        evaluator: &mut crate::evaluator::Evaluator,
        mut args: Vec<Value>,
        cursor: Cursor
    ) -> EvalResult<Value> {
        if let Some(bind) = &self.bind {
            args.insert(0, bind.clone());
        }
        self.callable.call(evaluator, args, cursor)
    }
}

impl Method {
    pub fn bind(self, val: Value) -> Method {
        if let Value::ObjInstance(_) = val {
            if let Method::User(func) = self {
                if let StmtKind::Fn { name, bound, .. } = func.declr.kind.clone() {
                    let env = Env::enclosed(func.closure.clone());
                    if bound || name == "init" {
                        env.borrow_mut().define("self".to_string(), val);
                    }
                    return Method::User(Function::new(func.declr, env, bound));
                }
                unreachable!();
            }

            if let Method::Native(func) = self {
                let bind = if func.bound { Some(val) } else { None };
                return Method::Native(NativeMethod {
                    callable: func.callable,
                    bound: true,
                    bind,
                });
            }
        }
        unreachable!("Non-obj value passed to Method::bind(val)");
    }

    pub fn get_callable(&self) -> Rc<dyn Callable> {
        return match self {
            Method::User(func) => Rc::new(func.clone()),
            Method::Native(func) => Rc::new(func.clone()),
        };
    }

    pub fn get_bound(&self) -> bool {
        return match self {
            Method::User(func) => func.bound,
            Method::Native(func) => func.bound,
        };
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    pub name: String,
    pub methods: HashMap<String, Method>,
}

impl Object {
    pub fn new(name: String, methods: HashMap<String, Method>) -> Self {
        Self { name, methods }
    }

    fn find_method(&self, name: String) -> Option<Method> {
        self.methods.get(&name).cloned()
    }
}

impl Callable for Object {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn arity(&self) -> usize {
        if let Some(init) = self.find_method("init".to_string()) {
            return match init {
                Method::User(func) => func.arity(),
                Method::Native(func) => func.arity(),
            };
        }

        0
    }

    fn call(
        &self,
        evaluator: &mut super::Evaluator,
        args: Vec<super::value::Value>,
        cursor: Cursor
    ) -> EvalResult<Value> {
        let inst = Value::ObjInstance(Rc::new(RefCell::new(Instance::new(self.clone()))));

        if let Some(init) = self.find_method("init".to_string()) {
            init.bind(inst.clone())
                .get_callable()
                .call(evaluator, args, cursor)?;
        }

        Ok(inst)
    }
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub obj: Object,
    fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(obj: Object) -> Self {
        Self {
            obj,
            fields: HashMap::new(),
        }
    }

    pub fn get_rc(
        inst_rc: Rc<RefCell<Instance>>,
        name: String,
        cursor: Cursor,
    ) -> EvalResult<Value> {
        let inst_ref = inst_rc.borrow();

        if let Some(val) = inst_ref.fields.get(&name) {
            return Ok(val.clone());
        }

        if let Some(method) = inst_ref.obj.find_method(name.clone()) {
            let bound = method.bind(Value::ObjInstance(inst_rc.clone()));
            return Ok(Value::Callable(bound.get_callable()));
        }

        Err(RuntimeEvent::error(
            format!("undefined property '{}'", name),
            cursor,
        ))
    }

    pub fn set(&mut self, name: String, val: Value) {
        self.fields.insert(name, val);
    }
}

impl ToString for Instance {
    fn to_string(&self) -> String {
        format!("{} instance", self.obj.name)
    }
}
