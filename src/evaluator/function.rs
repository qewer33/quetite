use crate::{
    evaluator::{
        Evaluator,
        env::{Env, EnvPtr},
        runtime_err::RuntimeEvent,
        value::{Callable, Value},
    },
    parser::stmt::{Stmt, StmtKind},
};

#[derive(Debug)]
pub struct Function {
    declr: Stmt,
    closure: EnvPtr,
}

impl Function {
    pub fn new(declr: Stmt, closure: EnvPtr) -> Self {
        if let StmtKind::Fn {
            name: _,
            params: _,
            body: _,
        } = declr.clone().kind
        {
            return Self { declr, closure };
        }

        unreachable!("Non-fn statement passed as declaration to Function::new(declr)");
    }
}

impl Callable for Function {
    fn name(&self) -> &str {
        if let StmtKind::Fn {
            name,
            params: _,
            body: _,
        } = &self.declr.kind
        {
            return name;
        }

        unreachable!("Non-fn statement passed as declaration to Function::new(declr)");
    }

    fn arity(&self) -> usize {
        if let StmtKind::Fn {
            name: _,
            params,
            body: _,
        } = &self.declr.kind
        {
            return params.len();
        }

        unreachable!("Non-fn statement passed as declaration to Function::new(declr)");
    }

    fn call(&self, evaluator: &mut Evaluator, args: Vec<Value>) -> Value {
        if let StmtKind::Fn {
            name: _,
            params,
            body,
        } = &self.declr.kind
        {
            let env = Env::enclosed(self.closure.clone());

            for (i, param) in params.iter().enumerate() {
                env.borrow_mut().define(param.clone(), args[i].clone());
            }

            match evaluator.eval_stmt_block(body, env) {
                Ok(()) => return Value::Null,
                Err(err) => {
                    // Catch the return value
                    if let RuntimeEvent::Return(val) = err {
                        return val;
                    }
                    return Value::Null;
                }
            }
        }

        unreachable!("Non-fn statement passed as declaration to Function::new(declr)");
    }
}
