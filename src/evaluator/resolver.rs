use std::collections::HashMap;

use crate::{
    lexer::{cursor::Cursor, token::KeywordKind},
    parser::{
        expr::{Expr, ExprKind},
        stmt::{Stmt, StmtKind},
    },
    reporter::Reporter,
    src::Src,
};

pub type ResolveResult = std::result::Result<(), ResolveErr>;

#[derive(Clone)]
pub struct ResolveErr {
    /// Error message
    pub msg: String,
    /// Error location as a Cursor
    pub cursor: Cursor,
}

impl ResolveErr {
    pub fn new(msg: String, cursor: Cursor) -> Self {
        Self { msg, cursor }
    }

    pub fn msg(mut self, msg: String) -> Self {
        self.msg = msg;
        self
    }

    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = cursor;
        self
    }
}

#[derive(Default, Clone)]
pub struct ResolverOutput {
    pub ast: Option<Vec<Stmt>>,
    pub errors: Option<Vec<ResolveErr>>,
    pub error_count: usize,
    pub warning_count: usize,
}

impl ResolverOutput {
    fn add_err(&mut self, error: ResolveErr) {
        if let None = self.errors {
            self.errors = Some(vec![]);
            self.ast = None;
        }
        if let Some(errors) = self.errors.as_mut() {
            errors.push(error);
            self.error_count += 1;
        }
    }
}

#[derive(Clone, Debug)]
struct ScopedVar {
    defined: bool,
    used: bool,
    loc: Cursor,
}

impl ScopedVar {
    fn declared(loc: Cursor) -> Self {
        ScopedVar {
            defined: false,
            used: false,
            loc,
        }
    }

    fn defined(loc: Cursor) -> Self {
        ScopedVar {
            defined: true,
            used: false,
            loc,
        }
    }
}

pub struct Resolver<'a> {
    pub src: &'a Src,
    pub ast: Vec<Stmt>,
    /// Stack of lexical scopes
    scopes: Vec<HashMap<String, ScopedVar>>,
    /// Resolver output
    out: ResolverOutput,
}

impl<'a> Resolver<'a> {
    pub fn new(src: &'a Src) -> Self {
        Self {
            src,
            ast: src.ast.clone().expect("expected ast"),
            scopes: vec![],
            out: ResolverOutput::default(),
        }
    }

    pub fn resolve(&mut self) -> ResolverOutput {
        let mut ast = self.ast.clone();
        for stmt in ast.iter_mut() {
            if let Err(err) = self.resolve_stmt(stmt) {
                self.out.add_err(err.clone());
                Reporter::error_at(&err.msg, "ResolveErr".into(), self.src, err.cursor);
            }
        }

        if self.out.error_count < 1 {
            self.out.ast = Some(ast);
        } else {
            self.out.ast = None;
        }
        self.out.clone()
    }

    // Statement functions

    fn resolve_stmts(&mut self, stmts: &Vec<Stmt>) -> ResolveResult {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> ResolveResult {
        match &stmt.kind {
            StmtKind::Expr(_) => self.resolve_stmt_expr(stmt),
            StmtKind::Throw(_) => self.resolve_stmt_err(stmt),
            StmtKind::Use(_) => self.resolve_stmt_use(stmt),
            StmtKind::Return(_) => self.resolve_stmt_return(stmt),
            StmtKind::Break => Ok(()),
            StmtKind::Continue => Ok(()),
            StmtKind::Var { .. } => self.resolve_stmt_var(stmt),
            StmtKind::Block(_) => self.resolve_stmt_block(stmt, false),
            StmtKind::If { .. } => self.resolve_stmt_if(stmt),
            StmtKind::Match { .. } => self.resolve_stmt_match(stmt),
            StmtKind::For { .. } => self.resolve_stmt_for(stmt),
            StmtKind::While { .. } => self.resolve_stmt_while(stmt),
            StmtKind::Try { .. } => self.resolve_stmt_try(stmt),
            StmtKind::Fn { .. } => self.resolve_stmt_fn(stmt),
            StmtKind::Obj { .. } => self.resolve_stmt_obj(stmt),
        }
    }

    fn resolve_stmt_block(&mut self, stmt: &Stmt, fn_block: bool) -> ResolveResult {
        if let StmtKind::Block(statements) = &stmt.kind {
            if !fn_block {
                self.begin_scope();
            }
            self.resolve_stmts(statements)?;
            if !fn_block {
                self.end_scope();
            }
            return Ok(());
        }
        unreachable!("Non-block statement passed to Resolver::resolve_stmt_block");
    }

    fn resolve_stmt_var(&mut self, stmt: &Stmt) -> ResolveResult {
        if let StmtKind::Var { name, init } = &stmt.kind {
            // Declare first (not defined yet) to catch self-initialization reads.
            self.declare(name.clone(), stmt.cursor);
            if let Some(expr) = init {
                self.resolve_expr(expr)?;
            }
            // Now make it visible/defined.
            self.define(name.clone(), stmt.cursor);
            return Ok(());
        }
        unreachable!("Non-var statement passed to Resolver::resolve_stmt_var");
    }

    fn resolve_stmt_expr(&mut self, stmt: &Stmt) -> ResolveResult {
        if let StmtKind::Expr(expr) = &stmt.kind {
            self.resolve_expr(expr)?;
            return Ok(());
        }
        unreachable!("Non-expr statement passed to Resolver::resolve_stmt_expr");
    }

    fn resolve_stmt_err(&mut self, stmt: &Stmt) -> ResolveResult {
        if let StmtKind::Throw(expr) = &stmt.kind {
            self.resolve_expr(expr)?;
            return Ok(());
        }
        unreachable!("Non-err statement passed to Resolver::resolve_stmt_err");
    }

    fn resolve_stmt_use(&mut self, stmt: &Stmt) -> ResolveResult {
        if let StmtKind::Use(expr) = &stmt.kind {
            self.resolve_expr(expr)?;
            return Ok(());
        }
        unreachable!("Non-use statement passed to Resolver::resolve_stmt_use");
    }

    fn resolve_stmt_return(&mut self, stmt: &Stmt) -> ResolveResult {
        if let StmtKind::Return(expr) = &stmt.kind {
            if let Some(e) = expr {
                self.resolve_expr(e)?;
            }
            return Ok(());
        }
        unreachable!("Non-return statement passed to Resolver::resolve_stmt_return");
    }

    fn resolve_stmt_if(&mut self, stmt: &Stmt) -> ResolveResult {
        if let StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } = &stmt.kind
        {
            self.resolve_expr(condition)?;
            self.resolve_stmt(then_branch)?;
            if let Some(else_s) = else_branch {
                self.resolve_stmt(else_s)?;
            }
            return Ok(());
        }
        unreachable!("Non-if statement passed to Resolver::resolve_stmt_if");
    }

    fn resolve_stmt_match(&mut self, stmt: &Stmt) -> ResolveResult {
        if let StmtKind::Match {
            val,
            arms,
            else_branch,
        } = &stmt.kind
        {
            self.resolve_expr(val)?;
            for (e, s) in arms.iter() {
                self.resolve_expr(e)?;
                self.resolve_stmt(s)?;
            }
            if let Some(else_s) = else_branch {
                self.resolve_stmt(else_s)?;
            }
            return Ok(());
        }
        unreachable!("Non-match statement passed to Resolver::resolve_stmt_match");
    }

    fn resolve_stmt_for(&mut self, stmt: &Stmt) -> ResolveResult {
        if let StmtKind::For {
            item,
            index,
            iter,
            body,
        } = &stmt.kind
        {
            self.resolve_expr(iter)?;

            // 2) loop body has its own scope
            self.begin_scope();

            // 3) declare+define the element variable
            self.declare(item.clone(), stmt.cursor);
            self.define(item.clone(), stmt.cursor);

            // 4) if there's an index variable, declare+define that too
            if let Some(idx_name) = index {
                self.declare(idx_name.clone(), stmt.cursor);
                self.define(idx_name.clone(), stmt.cursor);
            }

            // 5) resolve the body in that scope
            self.resolve_stmt_block(body, true)?;

            // 6) pop scope (will also warn on unused loop vars if you keep that)
            self.end_scope();

            return Ok(());
        }
        unreachable!("Non-for statement passed to Resolver::resolve_stmt_for");
    }

    fn resolve_stmt_while(&mut self, stmt: &Stmt) -> ResolveResult {
        if let StmtKind::While {
            declr,
            condition,
            step,
            body,
        } = &stmt.kind
        {
            if let Some(init_stmt) = declr {
                self.resolve_stmt(init_stmt)?;
            }
            self.resolve_expr(condition)?;
            if let Some(step_expr) = step {
                self.resolve_expr(step_expr)?;
            }
            self.resolve_stmt(body)?;
            return Ok(());
        }
        unreachable!("Non-while statement passed to Resolver::resolve_stmt_while");
    }

    fn resolve_stmt_try(&mut self, stmt: &Stmt) -> ResolveResult {
        if let StmtKind::Try {
            body,
            err_kind,
            err_val,
            catch,
            ensure,
        } = &stmt.kind
        {
            self.resolve_stmt(body)?;

            self.begin_scope();

            if let Some(kind) = err_kind {
                self.declare(kind.clone(), stmt.cursor);
                self.define(kind.clone(), stmt.cursor);
            }
            if let Some(val) = err_val {
                self.declare(val.clone(), stmt.cursor);
                self.define(val.clone(), stmt.cursor);
            }

            self.resolve_stmt_block(catch, true)?;

            self.end_scope();

            if let Some(ensure_body) = ensure {
                self.resolve_stmt(ensure_body)?;
            }

            return Ok(());
        }
        unreachable!("Non-try statement passed to Resolver::resolve_stmt_try");
    }

    fn resolve_stmt_fn(&mut self, stmt: &Stmt) -> ResolveResult {
        if let StmtKind::Fn {
            name, params, body, ..
        } = &stmt.kind
        {
            // Function name is bound in the enclosing scope.
            self.declare(name.clone(), stmt.cursor);
            self.define(name.clone(), stmt.cursor);

            // Resolve function body in its own scope with parameters.
            self.begin_scope();
            for p in params {
                self.declare(p.clone(), stmt.cursor);
                self.define(p.clone(), stmt.cursor);
            }
            self.resolve_stmt_block(body, true)?;
            self.end_scope();
            return Ok(());
        }
        unreachable!("Non-fn statement passed to Resolver::resolve_stmt_fn");
    }

    fn resolve_stmt_obj(&mut self, stmt: &Stmt) -> ResolveResult {
        if let StmtKind::Obj { name, methods } = &stmt.kind {
            self.declare(name.clone(), stmt.cursor);
            self.define(name.clone(), stmt.cursor);

            self.begin_scope();

            for method in methods {
                if let StmtKind::Fn { bound, .. } = &method.kind {
                    if *bound {
                        self.scopes.last_mut().unwrap().insert(
                            KeywordKind::KSelf.to_string(),
                            ScopedVar::defined(stmt.cursor),
                        );
                    }
                }
                self.resolve_stmt_fn(method)?;
            }

            self.end_scope();

            return Ok(());
        }
        unreachable!("Non-obj statement passed to Resolver::resolve_stmt_obj");
    }

    // Expression functions

    fn resolve_expr(&mut self, expr: &Expr) -> ResolveResult {
        match &expr.kind {
            ExprKind::Binary { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
                Ok(())
            }
            ExprKind::Ternary {
                condition,
                true_branch,
                false_branch,
            } => {
                self.resolve_expr(condition)?;
                self.resolve_expr(true_branch)?;
                self.resolve_expr(false_branch)?;
                Ok(())
            }
            ExprKind::Grouping { expr: inner } => {
                self.resolve_expr(inner)?;
                Ok(())
            }
            ExprKind::Unary { right, .. } => {
                self.resolve_expr(right)?;
                Ok(())
            }
            ExprKind::Literal(_) => Ok(()),
            ExprKind::List(list) => {
                for expr in list {
                    self.resolve_expr(expr)?;
                }
                Ok(())
            }
            ExprKind::Dict(dict) => {
                for (key, value) in dict {
                    self.resolve_expr(key)?;
                    self.resolve_expr(value)?;
                }
                Ok(())
            }
            ExprKind::Range {
                start, end, step, ..
            } => {
                self.resolve_expr(start)?;
                self.resolve_expr(end)?;
                if let Some(expr) = step {
                    self.resolve_expr(expr)?;
                }
                Ok(())
            }
            ExprKind::Index { obj, index } => {
                self.resolve_expr(obj)?;
                self.resolve_expr(index)?;
                Ok(())
            }
            ExprKind::IndexSet {
                obj, index, val, ..
            } => {
                self.resolve_expr(obj)?;
                self.resolve_expr(index)?;
                self.resolve_expr(val)?;
                Ok(())
            }
            ExprKind::Call { callee, args } => {
                self.resolve_expr(callee)?;
                for a in args {
                    self.resolve_expr(a)?;
                }
                Ok(())
            }
            ExprKind::Var(name) => self.resolve_expr_var(expr, name),
            ExprKind::Assign { name, val, .. } => {
                self.resolve_expr(val)?;
                self.resolve_local(expr, name);
                Ok(())
            }
            ExprKind::Logical { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
                Ok(())
            }
            ExprKind::Get { obj, .. } => {
                self.resolve_expr(obj)?;
                Ok(())
            }
            ExprKind::Set { obj, val, .. } => {
                self.resolve_expr(obj)?;
                self.resolve_expr(val)?;
                Ok(())
            }
            ExprKind::ESelf => {
                self.resolve_local(expr, KeywordKind::KSelf.to_string().as_str());
                Ok(())
            }
        }
    }

    fn resolve_expr_var(&mut self, expr: &Expr, name: &str) -> ResolveResult {
        // If the variable exists in the innermost scope but is not yet defined,
        // we’re reading it in its own initializer.
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(var) = scope.get_mut(name) {
                if !var.defined {
                    return Err(ResolveErr::new(
                        "can't read local variable in its own initializer".into(),
                        expr.cursor,
                    ));
                }
                if !var.used {
                    var.used = true;
                }
            }
        }

        // Annotate variable access distance if found; else it remains global (None).
        self.resolve_local(expr, name);
        Ok(())
    }

    // Utility functions

    fn resolve_local(&mut self, expr: &Expr, name: &str) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                *expr.resolved_dist.borrow_mut() = Some(i);
                return;
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        for (name, var) in self.scopes.last().unwrap() {
            if !var.used && *name != KeywordKind::KSelf.to_string() {
                Reporter::warning_at(
                    format!("local variable {} never used", name).as_str(),
                    self.src,
                    var.loc,
                );
                self.out.warning_count += 1;
            }
        }
        self.scopes.pop();
    }

    fn declare(&mut self, name: String, loc: Cursor) {
        if let Some(scope) = self.scopes.last_mut() {
            // false = declared but not yet defined
            scope.insert(name, ScopedVar::declared(loc));
        }
    }

    fn define(&mut self, name: String, loc: Cursor) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ScopedVar::defined(loc));
        }
    }
}
