use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::lexer::cursor::Cursor;
use crate::{
    evaluator::{
        Evaluator,
        env::EnvPtr,
        resolver::Resolver,
        runtime_err::{ErrKind, EvalResult, RuntimeEvent},
    },
    lexer::Lexer,
    parser::Parser,
    reporter::Reporter,
    src::Src,
};

pub type LoaderPtr = Rc<RefCell<Loader>>;

#[derive(Default)]
pub struct Loader {
    loaded: HashMap<PathBuf, EnvPtr>,
    visiting: HashSet<PathBuf>,
}

impl Loader {
    pub fn load(self_ptr: LoaderPtr, file: PathBuf, caller_dir: &Path) -> EvalResult<EnvPtr> {
        // Resolve path relative to caller and canonicalize for caching/cycle detection.
        let resolved = if file.is_absolute() {
            file
        } else {
            caller_dir.join(file)
        };
        let canonical = resolved.canonicalize().map_err(RuntimeEvent::from)?;

        // Fast path: already loaded.
        if let Some(env) = self_ptr.borrow().loaded.get(&canonical) {
            return Ok(env.clone());
        }

        // Detect cycles.
        if self_ptr.borrow().visiting.contains(&canonical) {
            return Err(RuntimeEvent::error(
                ErrKind::Value,
                format!("circular use of '{}'", canonical.display()),
                Cursor::new(),
            ));
        }

        // Mark visiting.
        self_ptr.borrow_mut().visiting.insert(canonical.clone());

        // Run the full pipeline (lex → parse → resolve → eval).
        let result = (|| -> EvalResult<EnvPtr> {
            let mut src = Src::new(canonical.clone());

            let mut lexer = Lexer::new(src.text.clone());
            src.tokens = Some(lexer.tokenize());

            let mut parser = Parser::new(&src);
            let parser_out = parser.parse();
            src.ast = match parser_out.ast {
                Some(ast) => {
                    if parser_out.warning_count > 0 {
                        Reporter::warning(
                            format!("parser exited with {} warnings", parser_out.warning_count)
                                .as_str(),
                        );
                    }
                    Some(ast)
                }
                None => {
                    return Err(RuntimeEvent::error(
                        ErrKind::Native,
                        format!("parser exited with {} errors", parser_out.error_count),
                        Cursor::new(),
                    ));
                }
            };

            let mut resolver = Resolver::new(&src);
            let resolver_out = resolver.resolve();
            src.ast = match resolver_out.ast {
                Some(ast) => {
                    if resolver_out.warning_count > 0 {
                        Reporter::warning(
                            format!(
                                "resolver exited with {} warnings",
                                resolver_out.warning_count
                            )
                            .as_str(),
                        );
                    }
                    Some(ast)
                }
                None => {
                    return Err(RuntimeEvent::error(
                        ErrKind::Native,
                        format!("resolver exited with {} errors", resolver_out.error_count),
                        Cursor::new(),
                    ));
                }
            };

            let mut evaluator = Evaluator::with_loader(&src, self_ptr.clone());
            evaluator.eval()?;

            Ok(evaluator.globals.clone())
        })();

        // Unmark visiting and cache on success.
        let mut loader = self_ptr.borrow_mut();
        loader.visiting.remove(&canonical);
        if let Ok(ref env) = result {
            loader.loaded.insert(canonical, env.clone());
        }

        result
    }
}
