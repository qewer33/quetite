pub mod env;
pub mod runtime_err;
pub mod value;

use crate::{
    evaluator::{
        env::Env,
        runtime_err::{EvalResult, RuntimeErr},
        value::Value,
    },
    parser::{
        expr::{BinaryOp, Expr, ExprKind, LiteralType, UnaryOp},
        stmt::{Stmt, StmtKind},
    },
    reporter::Reporter,
};

pub struct Evaluator {
    pub src: Vec<Stmt>,
    env: Env,
}

impl Evaluator {
    pub fn new(src: Vec<Stmt>) -> Self {
        Self {
            src,
            env: Env::new(),
        }
    }

    pub fn eval(&mut self) {
        for stmt in self.src.clone().iter() {
            match self.eval_stmt(stmt) {
                Ok(_) => {}
                Err(err) => {
                    Reporter::error(&err.msg);
                }
            }
        }
    }

    // Statement eval functions

    fn eval_stmt(&mut self, stmt: &Stmt) -> EvalResult<()> {
        match &stmt.kind {
            StmtKind::Expr(_) => self.eval_stmt_expr(stmt),
            StmtKind::Print(_) => self.eval_stmt_print(stmt),
            StmtKind::Var { name, init } => self.eval_stmt_var(stmt),
            StmtKind::Block(statements) => self.eval_stmt_block(stmt),
        }
    }

    fn eval_stmt_print(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::Print(expr) = &stmt.kind {
            let val = self.eval_expr(expr)?;
            println!("{}", val.to_string());

            return Ok(());
        }

        panic!("Non-print statement passed to Evaluator::eval_stmt_print");
    }

    fn eval_stmt_expr(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::Expr(expr) = &stmt.kind {
            self.eval_expr(expr)?;

            return Ok(());
        }

        panic!("Non-expr statement passed to Evaluator::eval_stmt_expr");
    }

    fn eval_stmt_var(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::Var { name, init } = &stmt.kind {
            let mut val = Value::Null;

            if let Some(expr) = init {
                val = self.eval_expr(expr)?;
            }

            self.env.define(name.clone(), val);

            return Ok(());
        }

        panic!("Non-var statement passed to Evaluator::eval_stmt_var");
    }

    fn eval_stmt_block(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::Block(statements) = &stmt.kind {
            let prev = self.env.clone();

            self.env = Env::with_enclosing(self.env.clone());

            for stmt in statements.iter() {
                self.eval_stmt(stmt)?;
            }

            self.env = prev;

            return Ok(());
        }

        panic!("Non-block statement passed to Evaluator::eval_stmt_block");
    }

    // Expression eval functions

    fn eval_expr(&mut self, expr: &Expr) -> EvalResult<Value> {
        match &expr.kind {
            ExprKind::Binary { left, op, right } => self.eval_expr_binary(expr),
            ExprKind::Grouping { expr: gexpr } => self.eval_expr_grouping(expr),
            ExprKind::Unary { op, right } => self.eval_expr_unary(expr),
            ExprKind::Literal(lit) => self.eval_expr_literal(expr),
            ExprKind::Var(name) => self.eval_expr_var(expr),
            ExprKind::Assign { name, val } => self.eval_expr_assign(expr),
        }
    }

    fn eval_expr_assign(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Assign { name, val } = &expr.kind {
            let val = self.eval_expr(val)?;
            self.env.define(name.clone(), val.clone());
            return Ok(val);
        }

        panic!("Non-assign passed to Evaluator::eval_expr_assign");
    }

    fn eval_expr_var(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Var(name) = &expr.kind {
            return Ok(self.env.get(name.clone())?);
        }

        panic!("Non-var passed to Evaluator::eval_expr_var");
    }

    fn eval_expr_literal(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Literal(literal) = &expr.kind {
            return match literal {
                LiteralType::Null => Ok(Value::Null),
                LiteralType::Int(i) => Ok(Value::Num(*i as f64)),
                LiteralType::Bool(b) => Ok(Value::Bool(*b)),
                LiteralType::Str(s) => Ok(Value::Str(s.clone())),
            };
        }

        panic!("Non-literal passed to Evaluator::eval_expr_literal");
    }

    fn eval_expr_grouping(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Grouping { expr } = &expr.kind {
            return Ok(self.eval_expr(expr)?);
        }

        dbg!(expr);
        panic!("Non-grouping passed to Evaluator::eval_expr_grouping");
    }

    fn eval_expr_unary(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Unary { op, right } = &expr.kind {
            let right = self.eval_expr(right)?;

            return match op {
                UnaryOp::Negate => Ok(Value::Num(-right.check_num()?)),
                UnaryOp::Not => Ok(Value::Bool(!right.is_truthy())),
            };
        }

        panic!("Non-unary passed to Evaluator::eval_expr_unary");
    }

    fn eval_expr_binary(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Binary { left, op, right } = &expr.kind {
            let left = self.eval_expr(left)?;
            let right = self.eval_expr(right)?;

            return match op {
                BinaryOp::Add => {
                    if let Value::Num(lnum) = left
                        && let Value::Num(rnum) = right
                    {
                        return Ok(Value::Num(lnum + rnum));
                    }

                    if let Value::Str(lstr) = left
                        && let Value::Str(rstr) = right
                    {
                        return Ok(Value::Str(format!("{}{}", lstr, rstr)));
                    }

                    return Ok(Value::Null);
                }
                BinaryOp::Sub => Ok(Value::Num(left.check_num()? - right.check_num()?)),
                BinaryOp::Mult => Ok(Value::Num(left.check_num()? * right.check_num()?)),
                BinaryOp::Div => Ok(Value::Num(left.check_num()? / right.check_num()?)),
                BinaryOp::Pow => Ok(Value::Num(left.check_num()?.powf(right.check_num()?))),
                BinaryOp::Equals => Ok(Value::Bool(left.is_equal(&right))),
                BinaryOp::NotEquals => Ok(Value::Bool(!left.is_equal(&right))),
                BinaryOp::Greater => Ok(Value::Bool(left.check_num()? > right.check_num()?)),
                BinaryOp::GreaterEquals => Ok(Value::Bool(left.check_num()? >= right.check_num()?)),
                BinaryOp::Lesser => Ok(Value::Bool(left.check_num()? < right.check_num()?)),
                BinaryOp::LesserEquals => Ok(Value::Bool(left.check_num()? <= right.check_num()?)),
            };
        }

        panic!("Non-binary passed to Evaluator::eval_expr_binary");
    }
}
