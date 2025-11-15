#[macro_export]
macro_rules! native_fn {
    ($name:ident, $str_name:expr, $arity:expr, |$evaluator:ident, $args:ident, $cursor:ident| $body:block) => {
        #[derive(Debug)]
        pub struct $name;
        impl Callable for $name {
            fn name(&self) -> &str {
                $str_name
            }
            fn arity(&self) -> usize {
                $arity
            }
            fn call(
                &self,
                $evaluator: &mut Evaluator,
                $args: Vec<Value>,
                $cursor: crate::lexer::cursor::Cursor,
            ) -> EvalResult<Value> {
                $body
            }
        }
    };
}

#[macro_export]
macro_rules! native_fn_with_data {
    ($struct_name:ident, $method_name:expr, $arity:expr, $data_type:ty, |$evaluator:ident, $args:ident, $cursor:ident, $data:ident| $body:block) => {
        struct $struct_name {
            data: Rc<RefCell<$data_type>>,
        }

        impl Callable for $struct_name {
            fn name(&self) -> &str {
                $method_name
            }
            fn arity(&self) -> usize {
                $arity
            }

            fn call(
                &self,
                $evaluator: &mut Evaluator,
                $args: Vec<Value>,
                $cursor: crate::lexer::cursor::Cursor,
            ) -> EvalResult<Value> {
                let $data = &self.data;
                $body
            }
        }

        impl std::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, stringify!($struct_name))
            }
        }
    };
}

#[macro_export]
macro_rules! native_fn_with_val {
    ($struct_name:ident, $method_name:expr, $arity:expr, $val_type:ty, |$evaluator:ident, $args:ident, $cursor:ident, $val:ident| $body:block) => {
        struct $struct_name {
            val: $val_type,
        }

        impl Callable for $struct_name {
            fn name(&self) -> &str {
                $method_name
            }
            fn arity(&self) -> usize {
                $arity
            }

            fn call(
                &self,
                $evaluator: &mut Evaluator,
                $args: Vec<Value>,
                $cursor: crate::lexer::cursor::Cursor,
            ) -> EvalResult<Value> {
                let $val = &self.val;
                $body
            }
        }

        impl std::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, stringify!($struct_name))
            }
        }
    };
}
