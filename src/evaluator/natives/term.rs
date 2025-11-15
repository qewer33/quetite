use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, Write},
    rc::Rc,
    time::Duration,
};

use crate::{
    evaluator::{
        Callable, EvalResult, Evaluator,
        object::{Method, NativeMethod, Object},
        value::Value,
    },
    native_fn, native_fn_with_data, native_fn_with_val,
};

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{Clear, ClearType, SetTitle, disable_raw_mode, enable_raw_mode},
};
use ordered_float::OrderedFloat;

pub fn native_term() -> Value {
    let mut methods: HashMap<String, Method> = HashMap::new();

    methods.insert(
        "size".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTermSize), false)),
    );
    methods.insert(
        "get_input".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTermGetInput), false)),
    );
    methods.insert(
        "cursor_hide".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTermCursorHide), false)),
    );
    methods.insert(
        "cursor_show".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTermCursorShow), false)),
    );
    methods.insert(
        "cursor_move".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTermCursorMove), false)),
    );
    methods.insert(
        "raw_enable".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTermRawEnable), false)),
    );
    methods.insert(
        "raw_disable".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTermRawDisable), false)),
    );
    methods.insert(
        "clear".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTermClear), false)),
    );
    methods.insert(
        "clear_line".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTermClearLine), false)),
    );
    methods.insert(
        "put".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTermPut), false)),
    );
    methods.insert(
        "write".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTermWrite), false)),
    );
    methods.insert(
        "set_title".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTermSetTitle), false)),
    );
    methods.insert(
        "flush".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTermFlush), false)),
    );

    Value::Obj(Rc::new(Object::new("Term".into(), methods)))
}

// Term.size() -> [width, height]: returns terminal dimensions
native_fn!(
    FnTermSize,
    "terminal_size",
    0,
    |_evaluator, _args, _cursor| {
        let (cols, rows) = crossterm::terminal::size()?;

        Ok(Value::List(Rc::new(RefCell::new(vec![
            Value::Num(OrderedFloat(cols as f64)),
            Value::Num(OrderedFloat(rows as f64)),
        ]))))
    }
);

native_fn!(
    FnTermGetInput,
    "terminal_get_input",
    0,
    |_evaluator, _args, _cursor| {
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = event::read()? {
                let key_str = match key_event.code {
                    KeyCode::BackTab => "Tab".into(),
                    _ => key_event.code.to_string(),
                };

                // Extract modifiers
                let ctrl = key_event.modifiers.contains(KeyModifiers::CONTROL);
                let shift = key_event.modifiers.contains(KeyModifiers::SHIFT)
                    || matches!(key_event.code, KeyCode::BackTab);
                let alt = key_event.modifiers.contains(KeyModifiers::ALT);

                // Create key data
                let key_data = Rc::new(RefCell::new(KeyInputData {
                    key: key_str,
                    ctrl,
                    shift,
                    alt,
                }));

                // Create methods
                let mut methods: HashMap<String, Method> = HashMap::new();

                methods.insert(
                    "key".into(),
                    Method::Native(NativeMethod::new(
                        Rc::new(KeyInputKeyGetter {
                            data: Rc::clone(&key_data),
                        }),
                        false,
                    )),
                );

                methods.insert(
                    "ctrl".into(),
                    Method::Native(NativeMethod::new(
                        Rc::new(KeyInputCtrlGetter { val: ctrl }),
                        false,
                    )),
                );

                methods.insert(
                    "shift".into(),
                    Method::Native(NativeMethod::new(
                        Rc::new(KeyInputShiftGetter { val: shift }),
                        false,
                    )),
                );

                methods.insert(
                    "alt".into(),
                    Method::Native(NativeMethod::new(
                        Rc::new(KeyInputAltGetter { val: alt }),
                        false,
                    )),
                );

                return Ok(Value::Obj(Rc::new(Object::new("KeyInput".into(), methods))));
            }
        }
        Ok(Value::Null)
    }
);

// Key input data structure
struct KeyInputData {
    key: String,
    ctrl: bool,
    shift: bool,
    alt: bool,
}

// Getter implementations using macros
native_fn_with_data!(
    KeyInputKeyGetter,
    "key",
    0,
    KeyInputData,
    |_evaluator, _args, _cursor, data| {
        let d = data.borrow();
        Ok(Value::Str(Rc::new(RefCell::new(d.key.clone()))))
    }
);

native_fn_with_val!(
    KeyInputCtrlGetter,
    "ctrl",
    0,
    bool,
    |_evaluator, _args, _cursor, val| { Ok(Value::Bool(*val)) }
);

native_fn_with_val!(
    KeyInputShiftGetter,
    "shift",
    0,
    bool,
    |_evaluator, _args, _cursor, val| { Ok(Value::Bool(*val)) }
);

native_fn_with_val!(
    KeyInputAltGetter,
    "alt",
    0,
    bool,
    |_evaluator, _args, _cursor, val| { Ok(Value::Bool(*val)) }
);

// Term.cursor_hide(): hides the cursor
native_fn!(
    FnTermCursorHide,
    "terminal_cursor_hide",
    0,
    |_evaluator, _args, _cursor| {
        execute!(io::stdout(), crossterm::cursor::Hide)?;
        Ok(Value::Null)
    }
);

// Term.cursor_show(): shows the cursor
native_fn!(
    FnTermCursorShow,
    "terminal_cursor_show",
    0,
    |_evaluator, _args, _cursor| {
        execute!(io::stdout(), crossterm::cursor::Show)?;
        Ok(Value::Null)
    }
);

// Term.cursor_move(x, y): moves cursor to position
native_fn!(
    FnTermCursorMove,
    "terminal_cursor_move",
    2,
    |_evaluator, args, _cursor| {
        let x = if let Value::Num(n) = args[0] {
            n.0 as u16
        } else {
            return Ok(Value::Null);
        };

        let y = if let Value::Num(n) = args[1] {
            n.0 as u16
        } else {
            return Ok(Value::Null);
        };

        execute!(io::stdout(), MoveTo(x, y))?;
        io::stdout().flush()?;
        Ok(Value::Null)
    }
);

// Term.raw_enable(): enables raw mode
native_fn!(
    FnTermRawEnable,
    "terminal_raw_enable",
    0,
    |_evaluator, _args, _cursor| {
        enable_raw_mode()?;
        Ok(Value::Null)
    }
);

// Term.raw_disable(): disables raw mode
native_fn!(
    FnTermRawDisable,
    "terminal_raw_disable",
    0,
    |_evaluator, _args, _cursor| {
        disable_raw_mode()?;
        Ok(Value::Null)
    }
);

// Term.clear(): clears entire screen and moves cursor to 0,0
native_fn!(
    FnTermClear,
    "terminal_clear",
    0,
    |_evaluator, _args, _cursor| {
        execute!(
            io::stdout(),
            Clear(ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )?;
        Ok(Value::Null)
    }
);

// Term.clear_line(): clears current line
native_fn!(
    FnTermClearLine,
    "terminal_clear_line",
    0,
    |_evaluator, _args, _cursor| {
        execute!(io::stdout(), Clear(ClearType::CurrentLine))?;
        io::stdout().flush()?;
        Ok(Value::Null)
    }
);

// Term.put(x, y, str): puts string at position without moving cursor after
native_fn!(FnTermPut, "terminal_put", 3, |_evaluator, args, _cursor| {
    let x = if let Value::Num(n) = args[0] {
        n.0 as u16
    } else {
        return Ok(Value::Null);
    };

    let y = if let Value::Num(n) = args[1] {
        n.0 as u16
    } else {
        return Ok(Value::Null);
    };

    let s = match &args[2] {
        Value::Str(s) => s.borrow().clone(),
        _ => " ".to_string(),
    };

    execute!(io::stdout(), MoveTo(x, y))?;
    print!("{s}");
    io::stdout().flush()?;

    Ok(Value::Null)
});

// Term.write(str): writes string at current cursor position and advances cursor
native_fn!(
    FnTermWrite,
    "terminal_write",
    1,
    |_evaluator, args, _cursor| {
        let s = match &args[0] {
            Value::Str(s) => s.borrow().clone(),
            other => other.to_string(),
        };

        print!("{s}");
        io::stdout().flush()?;

        Ok(Value::Null)
    }
);

// Term.set_title(str): sets terminal window title
native_fn!(
    FnTermSetTitle,
    "terminal_set_title",
    1,
    |_evaluator, args, _cursor| {
        if let Value::Str(s) = &args[0] {
            execute!(io::stdout(), SetTitle(s.borrow().as_str()))?;
        }
        Ok(Value::Null)
    }
);

// Term.flush(): manually flush stdout buffer
native_fn!(
    FnTermFlush,
    "terminal_flush",
    0,
    |_evaluator, _args, _cursor| {
        io::stdout().flush()?;
        Ok(Value::Null)
    }
);
