use crate::{
    evaluator::{
        Evaluator,
        env::{Env, EnvPtr},
        runtime_err::{EvalResult, RuntimeEvent},
        value::{Callable, Value},
    },
    parser::stmt::{Stmt, StmtKind},
};

#[derive(Debug, Clone)]
pub struct Function {
    pub declr: Stmt,
    pub closure: EnvPtr,
    pub bound: bool,
}

impl Function {
    pub fn new(declr: Stmt, closure: EnvPtr, bound: bool) -> Self {
        if let StmtKind::Fn { .. } = declr.clone().kind {
            return Self {
                declr,
                closure,
                bound,
            };
        }
        unreachable!("Non-fn statement passed as declaration to Function::new(declr)");
    }

    pub fn bind_method(self, val: Value) -> Function {
        if let Value::ObjInstance(_) = val {
            if let StmtKind::Fn { name, bound, .. } = self.declr.kind.clone() {
                let env = Env::enclosed(self.closure.clone());
                if bound || name == "init" {
                    env.borrow_mut().define("self".to_string(), val);
                }
                return Function::new(self.declr, env, bound);
            }
        }
        unreachable!("Non-obj value passed to Function::bind(val)");
    }
}

impl Callable for Function {
    fn name(&self) -> &str {
        if let StmtKind::Fn { name, .. } = &self.declr.kind {
            return name;
        }

        unreachable!("Non-fn statement passed as declaration to Function::new(declr)");
    }

    fn arity(&self) -> usize {
        if let StmtKind::Fn { params, .. } = &self.declr.kind {
            return params.len();
        }

        unreachable!("Non-fn statement passed as declaration to Function::new(declr)");
    }

    fn call(&self, evaluator: &mut Evaluator, args: Vec<Value>) -> EvalResult<Value> {
        if let StmtKind::Fn { params, body, .. } = &self.declr.kind {
            let env = Env::enclosed(self.closure.clone());

            for (i, param) in params.iter().enumerate() {
                env.borrow_mut().define(param.clone(), args[i].clone());
            }

            return match evaluator.eval_stmt_block(body, env) {
                Ok(()) => Ok(Value::Null),
                Err(RuntimeEvent::Return(v)) => Ok(v), // function return
                Err(e) => Err(e),
            };
        }

        unreachable!("Non-fn statement passed as declaration to Function::new(declr)");
    }
}
