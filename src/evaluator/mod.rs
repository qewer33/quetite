pub mod env;
pub mod function;
pub mod natives;
pub mod resolver;
pub mod runtime_err;
pub mod value;

use std::rc::Rc;

use crate::{
    evaluator::{
        env::{Env, EnvPtr},
        function::Function,
        natives::Natives,
        runtime_err::{EvalResult, RuntimeErr, RuntimeEvent},
        value::Value,
    },
    parser::{
        expr::{AssignOp, BinaryOp, Expr, ExprKind, LiteralType, LogicalOp, UnaryOp},
        stmt::{Stmt, StmtKind},
    },
    reporter::Reporter,
    src::Src,
};

pub struct Evaluator<'a> {
    pub src: &'a Src,
    ast: Vec<Stmt>,
    globals: EnvPtr,
    env: EnvPtr,
}

impl<'a> Evaluator<'a> {
    pub fn new(src: &'a Src) -> Self {
        let globals = Natives::get_natives();

        let mut this = Self {
            src,
            ast: src.ast.clone().expect("expected ast"),
            globals,
            env: Env::new(),
        };
        this.env = this.globals.clone();
        this
    }

    pub fn eval(&mut self) {
        for stmt in self.ast.clone().iter() {
            match self.eval_stmt(stmt) {
                Ok(_) => {}
                Err(err) => {
                    if let RuntimeEvent::Err(RuntimeErr { msg, cursor, .. }) = err {
                        Reporter::error_at(&msg, self.src, cursor);
                        return;
                    }
                }
            }
        }
    }

    // Statement functions

    fn eval_stmt(&mut self, stmt: &Stmt) -> EvalResult<()> {
        match &stmt.kind {
            StmtKind::Expr(_) => self.eval_stmt_expr(stmt),
            StmtKind::Return(_) => self.eval_stmt_return(stmt),
            StmtKind::Break => self.eval_stmt_break(stmt),
            StmtKind::Continue => self.eval_stmt_continue(stmt),
            StmtKind::Var { .. } => self.eval_stmt_var(stmt),
            StmtKind::Block(_) => self.eval_stmt_block(stmt, Env::enclosed(self.env.clone())),
            StmtKind::If { .. } => self.eval_stmt_if(stmt),
            StmtKind::While { .. } => self.eval_stmt_while(stmt),
            StmtKind::Fn { .. } => self.eval_stmt_fn(stmt),
        }
    }

    fn eval_stmt_return(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::Return(expr) = &stmt.kind {
            let mut val = Value::Null;
            if let Some(expr) = expr {
                val = self.eval_expr(expr)?;
            }
            return Err(RuntimeEvent::Return(val));
        }
        unreachable!("Non-return statement passed to Evaluator::eval_stmt_return");
    }

    fn eval_stmt_break(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::Break = &stmt.kind {
            return Err(RuntimeEvent::Break);
        }
        unreachable!("Non-break statement passed to Evaluator::eval_stmt_break");
    }

    fn eval_stmt_continue(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::Continue = &stmt.kind {
            return Err(RuntimeEvent::Continue);
        }
        unreachable!("Non-continue statement passed to Evaluator::eval_stmt_continue");
    }

    fn eval_stmt_if(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } = &stmt.kind
        {
            if self.eval_expr(condition)?.is_truthy() {
                self.eval_stmt(then_branch)?;
            } else if let Some(stmt) = else_branch {
                self.eval_stmt(stmt)?;
            }
            return Ok(());
        }
        unreachable!("Non-if statement passed to Evaluator::eval_stmt_if");
    }

    fn eval_stmt_while(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::While {
            declr,
            condition,
            step,
            body,
        } = &stmt.kind
        {
            if let Some(stmt) = declr {
                self.eval_stmt(stmt)?;
            }

            while self.eval_expr(condition)?.is_truthy() {
                match self.eval_stmt(body) {
                    Ok(_) => {}
                    Err(err) if err.is_continue() => {
                        if let Some(expr) = step {
                            self.eval_expr(expr)?;
                        }
                        continue;
                    }
                    Err(err) if err.is_break() => break,
                    Err(err) => return Err(err),
                }

                if let Some(expr) = step {
                    self.eval_expr(expr)?;
                }
            }

            return Ok(());
        }
        unreachable!("Non-while statement passed to Evaluator::eval_stmt_while");
    }

    fn eval_stmt_expr(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::Expr(expr) = &stmt.kind {
            self.eval_expr(expr)?;
            return Ok(());
        }
        unreachable!("Non-expr statement passed to Evaluator::eval_stmt_expr");
    }

    fn eval_stmt_var(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::Var { name, init } = &stmt.kind {
            let mut val = Value::Null;
            if let Some(expr) = init {
                val = self.eval_expr(expr)?;
            }
            self.env.borrow_mut().define(name.clone(), val);
            return Ok(());
        }
        unreachable!("Non-var statement passed to Evaluator::eval_stmt_var");
    }

    fn eval_stmt_fn(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::Fn { name, .. } = &stmt.kind {
            let function = Value::Callable(Rc::new(Function::new(stmt.clone(), self.env.clone())));
            self.env.borrow_mut().define(name.clone(), function);
            return Ok(());
        }
        unreachable!("Non-fn statement passed to Evaluator::eval_stmt_fn");
    }

    fn eval_stmt_block(&mut self, stmt: &Stmt, env: EnvPtr) -> EvalResult<()> {
        if let StmtKind::Block(statements) = &stmt.kind {
            let prev = self.env.clone();
            self.env = env;

            // save result
            let result = (|| -> EvalResult<()> {
                for s in statements {
                    self.eval_stmt(s)?;
                }
                Ok(())
            })();

            self.env = prev; // restore env
            return result; // propagate result
        }
        unreachable!("Non-block statement passed to Evaluator::eval_stmt_block");
    }

    // Expression functions

    #[rustfmt::skip]
    fn eval_expr(&mut self, expr: &Expr) -> EvalResult<Value> {
        match &expr.kind {
            ExprKind::Binary   { .. } => self.eval_expr_binary(expr),
            ExprKind::Grouping { .. } => self.eval_expr_grouping(expr),
            ExprKind::Unary    { .. } => self.eval_expr_unary(expr),
            ExprKind::Literal  (_)  => self.eval_expr_literal(expr),
            ExprKind::Call     { .. } => self.eval_expr_call(expr),
            ExprKind::Var      (_)  => self.eval_expr_var(expr),
            ExprKind::Assign   { .. } => self.eval_expr_assign(expr),
            ExprKind::Logical  { .. } => self.eval_expr_logical(expr),
        }
    }

    fn eval_expr_assign(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Assign { name, op, val } = &expr.kind {
            let mut val = self.eval_expr(val)?;

            if let Value::Num(mut num) = val {
                let var_val = self.lookup_var(name.clone(), expr)?;
                if let Value::Num(var_num) = var_val {
                    if let AssignOp::Add = op {
                        num += var_num;
                    }
                    if let AssignOp::Sub = op {
                        num = var_num - num;
                    }
                    val = Value::Num(num);
                }
            }

            if let Some(d) = expr.get_resolved_dist() {
                Env::assign_at(&self.env, name, val.clone(), d)?;
            } else {
                self.globals
                    .borrow_mut()
                    .assign(name, val.clone(), expr.cursor)?;
            }

            return Ok(val);
        }
        unreachable!("Non-assign passed to Evaluator::eval_expr_assign");
    }

    fn eval_expr_var(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Var(name) = &expr.kind {
            return self.lookup_var(name.clone(), expr);
        }
        unreachable!("Non-var passed to Evaluator::eval_expr_var");
    }

    fn eval_expr_logical(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Logical { left, op, right } = &expr.kind {
            let left = self.eval_expr(left)?;
            if let LogicalOp::Or = op {
                if left.is_truthy() {
                    return Ok(left);
                }
            } else {
                if !left.is_truthy() {
                    return Ok(left);
                }
            }
            return Ok(self.eval_expr(right)?);
        }
        unreachable!("Non-logical passed to Evaluator::eval_expr_logical");
    }

    fn eval_expr_literal(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Literal(literal) = &expr.kind {
            return match literal {
                LiteralType::Null => Ok(Value::Null),
                LiteralType::Num(i) => Ok(Value::Num(*i)),
                LiteralType::Bool(b) => Ok(Value::Bool(*b)),
                LiteralType::Str(s) => Ok(Value::Str(s.clone())),
            };
        }
        unreachable!("Non-literal passed to Evaluator::eval_expr_literal");
    }

    fn eval_expr_call(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Call { callee, args } = &expr.kind {
            let callee = self.eval_expr(callee)?;
            let mut args_values = Vec::with_capacity(args.len());
            for arg in args {
                args_values.push(self.eval_expr(arg)?);
            }

            if let Value::Callable(c) = callee {
                if args_values.len() != c.arity() {
                    return Err(RuntimeEvent::error(
                        format!(
                            "functions expects {} arguments but got {}",
                            c.arity(),
                            args_values.len()
                        ),
                        expr.cursor,
                    ));
                }
                return Ok(c.call(self, args_values)?);
            }
            return Err(RuntimeEvent::error(
                "can't call non-function".into(),
                expr.cursor,
            ));
        }
        unreachable!("Non-call passed to Evaluator::eval_expr_call");
    }

    fn eval_expr_grouping(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Grouping { expr } = &expr.kind {
            return Ok(self.eval_expr(expr)?);
        }
        unreachable!("Non-grouping passed to Evaluator::eval_expr_grouping");
    }

    fn eval_expr_unary(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Unary { op, right } = &expr.kind {
            let right = self.eval_expr(right)?;
            return match op {
                UnaryOp::Negate => Ok(Value::Num(-right.check_num(expr.cursor)?)),
                UnaryOp::Not => Ok(Value::Bool(!right.is_truthy())),
            };
        }
        unreachable!("Non-unary passed to Evaluator::eval_expr_unary");
    }

    fn eval_expr_binary(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Binary { left, op, right } = &expr.kind {
            let left = self.eval_expr(left)?;
            let right = self.eval_expr(right)?;
            let cursor = expr.cursor;

            return match op {
                BinaryOp::Add => {
                    if let (Value::Num(ln), Value::Num(rn)) = (left.clone(), right.clone()) {
                        Ok(Value::Num(ln + rn))
                    } else if let (Value::Str(ls), Value::Str(rs)) = (left, right) {
                        Ok(Value::Str(format!("{}{}", ls, rs)))
                    } else {
                        Ok(Value::Null)
                    }
                }
                BinaryOp::Sub => Ok(Value::Num(
                    left.check_num(cursor)? - right.check_num(cursor)?,
                )),
                BinaryOp::Mult => Ok(Value::Num(
                    left.check_num(cursor)? * right.check_num(cursor)?,
                )),
                BinaryOp::Div => Ok(Value::Num(
                    left.check_num(cursor)? / right.check_num(cursor)?,
                )),
                BinaryOp::Mod => Ok(Value::Num(
                    left.check_num(cursor)? % right.check_num(cursor)?,
                )),
                BinaryOp::Pow => Ok(Value::Num(
                    left.check_num(cursor)?.powf(right.check_num(cursor)?),
                )),
                BinaryOp::Equals => Ok(Value::Bool(left.is_equal(&right))),
                BinaryOp::NotEquals => Ok(Value::Bool(!left.is_equal(&right))),
                BinaryOp::Greater => Ok(Value::Bool(
                    left.check_num(cursor)? > right.check_num(cursor)?,
                )),
                BinaryOp::GreaterEquals => Ok(Value::Bool(
                    left.check_num(cursor)? >= right.check_num(cursor)?,
                )),
                BinaryOp::Lesser => Ok(Value::Bool(
                    left.check_num(cursor)? < right.check_num(cursor)?,
                )),
                BinaryOp::LesserEquals => Ok(Value::Bool(
                    left.check_num(cursor)? <= right.check_num(cursor)?,
                )),
                BinaryOp::Nullish => {
                    if let Value::Null = left {
                        Ok(right)
                    } else {
                        Ok(left)
                    }
                }
            };
        }
        unreachable!("Non-binary passed to Evaluator::eval_expr_binary");
    }

    // Utility functions

    pub fn lookup_var(&self, name: String, expr: &Expr) -> EvalResult<Value> {
        if let Some(d) = expr.get_resolved_dist() {
            Env::get_at(&self.env.clone(), name.as_str(), d, expr.cursor)
        } else {
            self.env.borrow().get(name.as_str(), expr.cursor)
        }
    }
}
