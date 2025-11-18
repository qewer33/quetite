pub mod env;
pub mod function;
pub mod loader;
pub mod natives;
pub mod object;
pub mod prototype;
pub mod resolver;
pub mod runtime_err;
pub mod value;

use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

use ordered_float::OrderedFloat;

use crate::{
    evaluator::{
        env::{Env, EnvPtr},
        function::Function,
        loader::{Loader, LoaderPtr},
        natives::Natives,
        object::{Instance, Method, Object},
        prototype::{BoundMethod, ValuePrototypes},
        runtime_err::{ErrKind, EvalResult, RuntimeErr, RuntimeEvent},
        value::{Callable, Value},
    },
    lexer::token::KeywordKind,
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
    prototypes: ValuePrototypes,
    loader: LoaderPtr,
}

impl<'a> Evaluator<'a> {
    pub fn new(src: &'a Src) -> Self {
        let globals = Natives::get_natives();

        let mut this = Self {
            src,
            ast: src.ast.clone().expect("expected ast"),
            globals,
            env: Env::new(),
            prototypes: ValuePrototypes::new(),
            loader: Rc::new(RefCell::new(Loader::default())),
        };
        this.env = this.globals.clone();
        this
    }

    pub fn with_loader(src: &'a Src, loader: LoaderPtr) -> Self {
        let mut evaluator = Evaluator::new(src);
        evaluator.loader = loader;
        evaluator
    }

    pub fn eval(&mut self) -> EvalResult<()> {
        for stmt in self.ast.clone().iter() {
            match self.eval_stmt(stmt) {
                Ok(_) => {}
                Err(err) => {
                    if let RuntimeEvent::Err(RuntimeErr {
                        kind, msg, cursor, ..
                    }) = &err
                    {
                        Reporter::error_at(msg, kind.to_string(), self.src, *cursor);
                    }
                    if let RuntimeEvent::UserErr { val, cursor } = &err {
                        let msg = format!("user error: {}", val);
                        Reporter::error_at(msg.as_str(), "UserErr".into(), self.src, *cursor);
                    }
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    // Statement functions

    fn eval_stmt(&mut self, stmt: &Stmt) -> EvalResult<()> {
        match &stmt.kind {
            StmtKind::Expr(_) => self.eval_stmt_expr(stmt),
            StmtKind::Throw(_) => self.eval_stmt_throw(stmt),
            StmtKind::Use(_) => self.eval_stmt_use(stmt),
            StmtKind::Return(_) => self.eval_stmt_return(stmt),
            StmtKind::Break => self.eval_stmt_break(stmt),
            StmtKind::Continue => self.eval_stmt_continue(stmt),
            StmtKind::Var { .. } => self.eval_stmt_var(stmt),
            StmtKind::Block(_) => self.eval_stmt_block(stmt, Env::enclosed(self.env.clone())),
            StmtKind::If { .. } => self.eval_stmt_if(stmt),
            StmtKind::For { .. } => self.eval_stmt_for(stmt),
            StmtKind::While { .. } => self.eval_stmt_while(stmt),
            StmtKind::Try { .. } => self.eval_stmt_try(stmt),
            StmtKind::Fn { .. } => self.eval_stmt_fn(stmt),
            StmtKind::Obj { .. } => self.eval_stmt_obj(stmt),
        }
    }

    fn eval_stmt_throw(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::Throw(expr) = &stmt.kind {
            let val = self.eval_expr(expr)?;
            return Err(RuntimeEvent::user_err(val, stmt.cursor));
        }
        unreachable!("Non-throw statement passed to Evaluator::eval_stmt_throw");
    }

    fn eval_stmt_use(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::Use(expr) = &stmt.kind {
            let val = self.eval_expr(expr)?;
            let path_rc = val.check_str(stmt.cursor, Some("use path".into()))?;
            let path_str = path_rc.borrow().clone();

            // Resolve relative to current source file.
            let caller_dir = self.src.file.parent().unwrap_or_else(|| Path::new("."));

            match Loader::load(self.loader.clone(), PathBuf::from(path_str), caller_dir) {
                Ok(env) => {
                    // Merge imported globals into our globals.
                    for (name, value) in env.borrow().entries() {
                        self.globals.borrow_mut().define(name, value);
                    }

                    return Ok(());
                }
                Err(_) => {
                    return Err(RuntimeEvent::error(
                        ErrKind::IO,
                        "failed to load file".into(),
                        stmt.cursor,
                    ));
                }
            }
        }
        unreachable!("Non-continue statement passed to Evaluator::eval_stmt_continue");
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

    fn eval_stmt_for(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::For {
            item,
            index,
            iter,
            body,
        } = &stmt.kind
        {
            let iter = self.eval_expr(&iter)?;

            match iter {
                Value::List(rc_list) => {
                    let len = rc_list.borrow().len();

                    for i in 0..len {
                        let elem = rc_list.borrow()[i].clone();

                        let loop_env = Env::enclosed(self.env.clone());
                        loop_env.borrow_mut().define(item.clone(), elem);

                        if let Some(idx_name) = index {
                            loop_env
                                .borrow_mut()
                                .define(idx_name.clone(), Value::Num(OrderedFloat(i as f64)));
                        }

                        match self.eval_stmt_block(body, loop_env) {
                            Ok(_) => {}
                            Err(err) if err.is_continue() => continue,
                            Err(err) if err.is_break() => break,
                            Err(err) => return Err(err),
                        }
                    }
                }
                Value::Str(rc_str) => {
                    let chars: Vec<char> = rc_str.borrow().chars().collect();
                    for (i, ch) in chars.into_iter().enumerate() {
                        let loop_env = Env::enclosed(self.env.clone());
                        loop_env.borrow_mut().define(
                            item.clone(),
                            Value::Str(Rc::new(RefCell::new(ch.to_string()))),
                        );
                        if let Some(idx_name) = index {
                            loop_env
                                .borrow_mut()
                                .define(idx_name.clone(), Value::Num(OrderedFloat(i as f64)));
                        }
                        match self.eval_stmt_block(body, loop_env) {
                            Ok(_) => {}
                            Err(err) if err.is_continue() => continue,
                            Err(err) if err.is_break() => break,
                            Err(err) => return Err(err),
                        }
                    }
                }
                _ => {
                    return Err(RuntimeEvent::error(
                        ErrKind::Type,
                        "only List and Str values are iterable".into(),
                        stmt.cursor,
                    ));
                }
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

    fn eval_stmt_try(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::Try {
            body,
            err_kind,
            err_val,
            catch,
            ensure,
        } = &stmt.kind
        {
            let out = match self.eval_stmt(body) {
                Err(e) => match e {
                    RuntimeEvent::UserErr { val, .. } => {
                        let catch_env = Env::enclosed(self.env.clone());
                        if let Some(kind) = err_kind {
                            catch_env.borrow_mut().define(
                                kind.clone(),
                                Value::Str(Rc::new(RefCell::new("UserErr".into()))),
                            );
                        }
                        if let Some(eval) = err_val {
                            catch_env.borrow_mut().define(eval.clone(), val);
                        }

                        self.eval_stmt_block(catch, catch_env)
                    }
                    RuntimeEvent::Err(err) => {
                        let catch_env = Env::enclosed(self.env.clone());
                        if let Some(kind) = err_kind {
                            catch_env.borrow_mut().define(
                                kind.clone(),
                                Value::Str(Rc::new(RefCell::new("RuntimeErr".into()))),
                            );
                        }
                        if let Some(eval) = err_val {
                            catch_env
                                .borrow_mut()
                                .define(eval.clone(), Value::Str(Rc::new(RefCell::new(err.msg))));
                        }

                        self.eval_stmt_block(catch, catch_env)
                    }
                    other => Err(other),
                },
                Ok(()) => Ok(()),
            };

            if let Some(ensure_body) = ensure {
                if let Err(e) = self.eval_stmt(ensure_body) {
                    return Err(e);
                }
            }

            return out;
        }
        unreachable!("Non-try statement passed to Evaluator::eval_stmt_try");
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
        if let StmtKind::Fn { name, bound, .. } = &stmt.kind {
            let func = Value::Callable(Rc::new(Function::new(
                stmt.clone(),
                self.env.clone(),
                *bound,
            )));
            self.env.borrow_mut().define(name.clone(), func);
            return Ok(());
        }
        unreachable!("Non-fn statement passed to Evaluator::eval_stmt_fn");
    }

    fn eval_stmt_obj(&mut self, stmt: &Stmt) -> EvalResult<()> {
        if let StmtKind::Obj { name, methods } = &stmt.kind {
            self.env.borrow_mut().define(name.clone(), Value::Null);

            let mut obj_methods: HashMap<String, Method> = HashMap::new();
            for method in methods.to_owned() {
                if let StmtKind::Fn { bound, .. } = &method.kind {
                    let func: Function = Function::new(method.clone(), self.env.clone(), *bound);
                    obj_methods.insert(func.name().to_string(), Method::User(func));
                }
            }

            self.env.borrow_mut().assign(
                name.as_str(),
                Value::Obj(Rc::new(Object::new(name.clone(), obj_methods))),
                stmt.cursor,
            )?;
            return Ok(());
        }
        unreachable!("Non-obj statement passed to Evaluator::eval_stmt_obj");
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

    fn eval_expr(&mut self, expr: &Expr) -> EvalResult<Value> {
        match &expr.kind {
            ExprKind::Binary { .. } => self.eval_expr_binary(expr),
            ExprKind::Ternary { .. } => self.eval_expr_ternary(expr),
            ExprKind::Grouping { .. } => self.eval_expr_grouping(expr),
            ExprKind::Unary { .. } => self.eval_expr_unary(expr),
            ExprKind::Literal(_) => self.eval_expr_literal(expr),
            ExprKind::List(_) => self.eval_expr_list(expr),
            ExprKind::Range { .. } => self.eval_expr_range(expr),
            ExprKind::Index { .. } => self.eval_expr_index(expr),
            ExprKind::IndexSet { .. } => self.eval_expr_index_set(expr),
            ExprKind::Call { .. } => self.eval_expr_call(expr),
            ExprKind::Var(_) => self.eval_expr_var(expr),
            ExprKind::Assign { .. } => self.eval_expr_assign(expr),
            ExprKind::Logical { .. } => self.eval_expr_logical(expr),
            ExprKind::Get { .. } => self.eval_expr_get(expr),
            ExprKind::Set { .. } => self.eval_expr_set(expr),
            ExprKind::ESelf => self.lookup_var(KeywordKind::KSelf.to_string().as_str(), expr),
        }
    }

    fn eval_expr_assign(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Assign { name, op, val } = &expr.kind {
            let rhs_val = self.eval_expr(val)?;

            // read current
            let current = self.lookup_var(name.as_str(), expr)?;

            // compute new value
            let new_val = match op {
                AssignOp::Value => rhs_val.clone(),
                AssignOp::Add => current.add_assign(rhs_val, expr.cursor)?,
                AssignOp::Sub => current.sub_assign(rhs_val, expr.cursor)?,
            };

            // write back
            if let Some(d) = expr.get_resolved_dist() {
                Env::assign_at(&self.env, name, new_val.clone(), d)?;
            } else {
                self.globals
                    .borrow_mut()
                    .assign(name, new_val.clone(), expr.cursor)?;
            }

            return Ok(new_val);
        }

        unreachable!("Non-assign passed to Evaluator::eval_expr_assign");
    }

    fn eval_expr_ternary(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Ternary {
            condition,
            true_branch,
            false_branch,
        } = &expr.kind
        {
            if self.eval_expr(&condition)?.is_truthy() {
                return self.eval_expr(&true_branch);
            } else {
                return self.eval_expr(&false_branch);
            }
        }
        unreachable!("Non-ternary passed to Evaluator::eval_expr_ternary");
    }

    fn eval_expr_var(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Var(name) = &expr.kind {
            return self.lookup_var(name.as_str(), expr);
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
                LiteralType::Num(i) => Ok(Value::Num(OrderedFloat(*i))),
                LiteralType::Bool(b) => Ok(Value::Bool(*b)),
                LiteralType::Str(s) => Ok(Value::Str(Rc::new(RefCell::new(s.clone())))),
            };
        }
        unreachable!("Non-literal passed to Evaluator::eval_expr_literal");
    }

    fn eval_expr_list(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::List(list) = &expr.kind {
            let mut values: Vec<Value> = vec![];

            for expr in list {
                values.push(self.eval_expr(expr)?);
            }

            return Ok(Value::List(Rc::new(RefCell::new(values))));
        }
        unreachable!("Non-list passed to Evaluator::eval_expr_list");
    }

    fn eval_expr_range(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Range {
            start,
            end,
            inclusive,
            step,
        } = &expr.kind
        {
            let mut values: Vec<Value> = vec![];

            let mut nstart: f64 = 0.0;
            let val = self.eval_expr(start)?;
            if let Value::Num(n) = val {
                nstart = n.0;
            } else {
                return Err(RuntimeEvent::error(
                    ErrKind::Type,
                    "range start must be a Num".into(),
                    expr.cursor,
                ));
            }

            let mut nend: f64 = 0.0;
            let val = self.eval_expr(end)?;
            if let Value::Num(n) = val {
                nend = n.0;
            } else {
                return Err(RuntimeEvent::error(
                    ErrKind::Type,
                    "range end must be a Num".into(),
                    expr.cursor,
                ));
            }

            let mut nstep: f64 = 1.0;
            if let Some(expr) = step {
                let val = self.eval_expr(expr)?;
                if let Value::Num(n) = val {
                    nstep = n.0;
                } else {
                    return Err(RuntimeEvent::error(
                        ErrKind::Type,
                        "range step must be a Num".into(),
                        expr.cursor,
                    ));
                }
            }

            let incr = nstart < nend;
            let mut i = nstart;
            if *inclusive {
                while i <= nend {
                    values.push(Value::Num(OrderedFloat(i)));
                    if incr { i += nstep } else { i -= nstep }
                }
            } else {
                while i < nend {
                    values.push(Value::Num(OrderedFloat(i)));
                    if incr { i += nstep } else { i -= nstep }
                }
            }

            return Ok(Value::List(Rc::new(RefCell::new(values))));
        }
        unreachable!("Non-range passed to Evaluator::eval_expr_range");
    }

    fn eval_expr_index(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Index { obj, index } = &expr.kind {
            let base_val = self.eval_expr(obj)?;
            let index_val = self.eval_expr(index)?;

            let idx = match index_val {
                Value::Num(n) => n.0 as usize,
                _ => {
                    return Err(RuntimeEvent::error(
                        ErrKind::Type,
                        "list index must be a Num".into(),
                        index.cursor,
                    ));
                }
            };

            return match base_val {
                Value::List(rc_items) => {
                    let items = rc_items.borrow();
                    if idx >= items.len() {
                        return Err(RuntimeEvent::error(
                            ErrKind::Value,
                            format!("list index {} out of bounds (len = {})", idx, items.len()),
                            expr.cursor,
                        ));
                    }
                    Ok(items[idx].clone())
                }
                Value::Str(s) => {
                    let chars: Vec<char> = s.borrow().chars().collect();
                    if idx >= chars.len() {
                        return Err(RuntimeEvent::error(
                            ErrKind::Value,
                            format!("string index {} out of bounds (len = {})", idx, chars.len()),
                            expr.cursor,
                        ));
                    }
                    Ok(Value::Str(Rc::new(RefCell::new(chars[idx].to_string()))))
                }
                _ => Err(RuntimeEvent::error(
                    ErrKind::Type,
                    "value is not indexable".into(),
                    expr.cursor,
                )),
            };
        }
        unreachable!("Non-index passed to eval_expr_index");
    }

    fn eval_expr_index_set(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::IndexSet {
            obj, index, val, ..
        } = &expr.kind
        {
            let base_val = self.eval_expr(obj)?;
            let index_val = self.eval_expr(index)?;

            let idx = match index_val {
                Value::Num(n) => n.0 as usize,
                _ => {
                    return Err(RuntimeEvent::error(
                        ErrKind::Type,
                        "list index must be a Num".into(),
                        index.cursor,
                    ));
                }
            };

            return match base_val {
                Value::List(items) => {
                    if idx >= items.borrow().len() {
                        return Err(RuntimeEvent::error(
                            ErrKind::Value,
                            format!(
                                "list index {} out of bounds (len = {})",
                                idx,
                                items.borrow().len()
                            ),
                            expr.cursor,
                        ));
                    }

                    let set_val = self.eval_expr(val)?;
                    items.borrow_mut()[idx] = set_val.clone();

                    Ok(set_val)
                }
                Value::Str(s) => {
                    let chars: Vec<char> = s.borrow().chars().collect();
                    if idx >= chars.len() {
                        return Err(RuntimeEvent::error(
                            ErrKind::Value,
                            format!("string index {} out of bounds (len = {})", idx, chars.len()),
                            expr.cursor,
                        ));
                    }

                    let set_val = self.eval_expr(val)?;
                    if let Value::Str(set_str) = set_val.clone() {
                        s.borrow_mut()
                            .replace_range(idx..=idx, set_str.borrow().as_str());
                        return Ok(set_val);
                    }

                    Err(RuntimeEvent::error(
                        ErrKind::Type,
                        "can't set index of Str to non-Str".into(),
                        expr.cursor,
                    ))
                }
                _ => Err(RuntimeEvent::error(
                    ErrKind::Type,
                    "value is not indexable".into(),
                    expr.cursor,
                )),
            };
        }
        unreachable!("Non-index_set passed to Evaluator::eval_index_set");
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
                        ErrKind::Arity,
                        format!(
                            "function expects {} arguments but got {}",
                            c.arity(),
                            args_values.len()
                        ),
                        expr.cursor,
                    ));
                }
                return Ok(c.call(self, args_values, expr.cursor)?);
            }

            if let Value::Obj(obj) = callee {
                if args_values.len() != obj.arity() {
                    return Err(RuntimeEvent::error(
                        ErrKind::Arity,
                        format!(
                            "object initializer expects {} arguments but got {}",
                            obj.arity(),
                            args_values.len()
                        ),
                        expr.cursor,
                    ));
                }
                return Ok(obj.call(self, args_values, expr.cursor)?);
            }

            return Err(RuntimeEvent::error(
                ErrKind::Type,
                "can only call functions or objects".into(),
                expr.cursor,
            ));
        }
        unreachable!("Non-call passed to Evaluator::eval_expr_call");
    }

    fn eval_expr_get(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Get { obj, name } = &expr.kind {
            let val = self.eval_expr(obj)?;

            // instance methods
            if let Value::ObjInstance(inst) = val {
                return Ok(Instance::get_rc(inst.clone(), name.clone(), expr.cursor)?);
            }

            // static methods
            if let Value::Obj(obj) = val {
                if let Some(method) = obj.methods.get(&name.clone()) {
                    if !method.get_bound() {
                        return Ok(Value::Callable(method.get_callable()));
                    } else {
                        return Err(RuntimeEvent::error(
                            ErrKind::Name,
                            format!(
                                "can't call bound method '{}' of object '{}' without an instance",
                                name, obj.name
                            ),
                            expr.cursor,
                        ));
                    }
                }
                return Err(RuntimeEvent::error(
                    ErrKind::Name,
                    format!("static method '{}' undefined in object {}", name, obj.name),
                    expr.cursor,
                ));
            }

            // primitive prototype methods
            if let Some(proto) = val.prototype(&self.prototypes) {
                if let Some(method) = proto.get_method(name.clone()) {
                    let bound = BoundMethod {
                        receiver: val.clone(),
                        method,
                    };
                    return Ok(Value::Callable(Rc::new(bound)));
                }
                return Err(RuntimeEvent::error(
                    ErrKind::Name,
                    format!("method '{}' not found in {} prototype", name, proto.name),
                    expr.cursor,
                ));
            }

            return Err(RuntimeEvent::error(
                ErrKind::Type,
                "only instances and primitives with prototypes have properties".into(),
                expr.cursor,
            ));
        }
        unreachable!("Non-get passed to Evaluator::eval_expr_get");
    }

    fn eval_expr_set(&mut self, expr: &Expr) -> EvalResult<Value> {
        if let ExprKind::Set { obj, name, op, val } = &expr.kind {
            let obj = self.eval_expr(obj)?;

            if let Value::ObjInstance(inst) = obj {
                let rhs_val = self.eval_expr(val)?;

                let new_val = match op {
                    AssignOp::Value => rhs_val.clone(),
                    AssignOp::Add => {
                        let current = Instance::get_rc(inst.clone(), name.clone(), expr.cursor)?;
                        current.add_assign(rhs_val, expr.cursor)?
                    }
                    AssignOp::Sub => {
                        let current = Instance::get_rc(inst.clone(), name.clone(), expr.cursor)?;
                        current.sub_assign(rhs_val, expr.cursor)?
                    }
                };

                inst.borrow_mut().set(name.clone(), new_val.clone());
                return Ok(new_val);
            }

            return Err(RuntimeEvent::error(
                ErrKind::Type,
                "only instances have fields".into(),
                expr.cursor,
            ));
        }
        unreachable!("Non-set passed to Evaluator::eval_expr_set");
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
                UnaryOp::Negate => Ok(Value::Num(OrderedFloat(
                    -right.check_num(expr.cursor, None)?,
                ))),
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
                        Ok(Value::Str(Rc::new(RefCell::new(format!(
                            "{}{}",
                            ls.borrow(),
                            rs.borrow()
                        )))))
                    } else {
                        Ok(Value::Null)
                    }
                }
                BinaryOp::Sub => Ok(Value::Num(OrderedFloat(
                    left.check_num(cursor, None)? - right.check_num(cursor, None)?,
                ))),
                BinaryOp::Mult => Ok(Value::Num(OrderedFloat(
                    left.check_num(cursor, None)? * right.check_num(cursor, None)?,
                ))),
                BinaryOp::Div => Ok(Value::Num(OrderedFloat(
                    left.check_num(cursor, None)? / right.check_num(cursor, None)?,
                ))),
                BinaryOp::Mod => Ok(Value::Num(OrderedFloat(
                    left.check_num(cursor, None)? % right.check_num(cursor, None)?,
                ))),
                BinaryOp::Pow => Ok(Value::Num(OrderedFloat(
                    left.check_num(cursor, None)?
                        .powf(right.check_num(cursor, None)?),
                ))),
                BinaryOp::Equals => Ok(Value::Bool(left.is_equal(&right))),
                BinaryOp::NotEquals => Ok(Value::Bool(!left.is_equal(&right))),
                BinaryOp::Greater => Ok(Value::Bool(
                    left.check_num(cursor, None)? > right.check_num(cursor, None)?,
                )),
                BinaryOp::GreaterEquals => Ok(Value::Bool(
                    left.check_num(cursor, None)? >= right.check_num(cursor, None)?,
                )),
                BinaryOp::Lesser => Ok(Value::Bool(
                    left.check_num(cursor, None)? < right.check_num(cursor, None)?,
                )),
                BinaryOp::LesserEquals => Ok(Value::Bool(
                    left.check_num(cursor, None)? <= right.check_num(cursor, None)?,
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

    pub fn lookup_var(&self, name: &str, expr: &Expr) -> EvalResult<Value> {
        if let Some(d) = expr.get_resolved_dist() {
            Env::get_at(&self.env.clone(), name, d, expr.cursor)
        } else {
            self.env.borrow().get(name, expr.cursor)
        }
    }
}
