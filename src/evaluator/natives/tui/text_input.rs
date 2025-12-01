use crate::{
    evaluator::{
        natives::tui::{TuiStyle, WIDGETS, Widget},
        object::{Method, NativeMethod, Object},
    },
    native_fn, native_fn_with_data,
};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::evaluator::{Callable, EvalResult, Evaluator, value::Value};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

// Tui.create_text_input(x, y, width, placeholder) -> TextInput object
native_fn!(
    FnTuiCreateTextInput,
    "tui_create_text_input",
    4,
    |_evaluator, args, cursor| {
        let x = args[0].check_num(cursor, Some("x position".into()))? as u16;
        let y = args[1].check_num(cursor, Some("y position".into()))? as u16;
        let width = args[2].check_num(cursor, Some("width".into()))? as u16;
        let placeholder = string_from_value(&args[3]);

        let input_data = Rc::new(RefCell::new(TextInputData {
            x,
            y,
            width,
            content: String::new(),
            cursor: 0,
            placeholder,
            focused: false,
            style: TuiStyle::default(),
        }));

        let mut methods: HashMap<String, Method> = HashMap::new();

        methods.insert(
            "get_text".into(),
            Method::Native(NativeMethod::new(
                Rc::new(TextInputGetTextMethod {
                    data: Rc::clone(&input_data),
                }),
                false,
            )),
        );

        methods.insert(
            "set_text".into(),
            Method::Native(NativeMethod::new(
                Rc::new(TextInputSetTextMethod {
                    data: Rc::clone(&input_data),
                }),
                false,
            )),
        );

        methods.insert(
            "handle_key".into(),
            Method::Native(NativeMethod::new(
                Rc::new(TextInputHandleKeyMethod {
                    data: Rc::clone(&input_data),
                }),
                false,
            )),
        );

        methods.insert(
            "clear".into(),
            Method::Native(NativeMethod::new(
                Rc::new(TextInputClearMethod {
                    data: Rc::clone(&input_data),
                }),
                false,
            )),
        );

        methods.insert(
            "set_focused".into(),
            Method::Native(NativeMethod::new(
                Rc::new(TextInputSetFocusedMethod {
                    data: Rc::clone(&input_data),
                }),
                false,
            )),
        );

        methods.insert(
            "set_style".into(),
            Method::Native(NativeMethod::new(
                Rc::new(TextInputSetStyleMethod {
                    data: Rc::clone(&input_data),
                }),
                false,
            )),
        );

        methods.insert(
            "render".into(),
            Method::Native(NativeMethod::new(
                Rc::new(TextInputRenderMethod {
                    data: Rc::clone(&input_data),
                }),
                false,
            )),
        );

        Ok(Value::Obj(Rc::new(Object::new(
            "TextInput".into(),
            methods,
        ))))
    }
);

fn string_from_value(value: &Value) -> String {
    match value {
        Value::Str(s) => s.borrow().clone(),
        _ => String::new(),
    }
}

#[derive(Clone)]
pub struct TextInputData {
    x: u16,
    y: u16,
    width: u16,
    content: String,
    cursor: usize,
    placeholder: String,
    focused: bool,
    style: TuiStyle,
}

// Method implementations using the macro

native_fn_with_data!(
    TextInputGetTextMethod,
    "get_text",
    0,
    TextInputData,
    |_evaluator, _args, _cursor, data| {
        let d = data.borrow();
        Ok(Value::Str(Rc::new(RefCell::new(d.content.clone()))))
    }
);

native_fn_with_data!(
    TextInputSetTextMethod,
    "set_text",
    1,
    TextInputData,
    |_evaluator, args, _cursor, data| {
        let text = match &args[0] {
            Value::Str(s) => s.borrow().clone(),
            _ => return Ok(Value::Null),
        };

        let mut d = data.borrow_mut();
        d.content = text;
        d.cursor = d.content.chars().count();

        Ok(Value::Null)
    }
);

native_fn_with_data!(
    TextInputHandleKeyMethod,
    "handle_key",
    1,
    TextInputData,
    |_evaluator, args, _cursor, data| {
        let key = match &args[0] {
            Value::Str(s) => s.borrow().clone(),
            _ => return Ok(Value::Null),
        };

        let mut d = data.borrow_mut();
        let cursor = d.cursor.clone();

        match key.as_str() {
            "Backspace" => {
                if cursor > 0 {
                    let mut chars: Vec<char> = d.content.chars().collect();
                    chars.remove(cursor - 1);
                    d.content = chars.into_iter().collect();
                    d.cursor -= 1;
                }
            }
            "Space" => {
                d.content.insert(cursor, ' ');
                d.cursor += 1;
            }
            "Delete" => {
                let char_count = d.content.chars().count();
                if cursor < char_count {
                    let mut chars: Vec<char> = d.content.chars().collect();
                    chars.remove(cursor);
                    d.content = chars.into_iter().collect();
                }
            }
            "Left" => {
                if cursor > 0 {
                    d.cursor -= 1;
                }
            }
            "Right" => {
                if cursor < d.content.chars().count() {
                    d.cursor += 1;
                }
            }
            "Home" => {
                d.cursor = 0;
            }
            "End" => {
                d.cursor = d.content.chars().count();
            }
            // Don't process special keys
            "Shift" | "Up" | "Down" | "Enter" | "Esc" | "Tab" | "PageUp" | "PageDown" => {}
            // Everything else is a printable character
            _ => {
                let mut chars: Vec<char> = d.content.chars().collect();
                for c in key.chars() {
                    chars.insert(cursor, c);
                    d.cursor += 1;
                }
                d.content = chars.into_iter().collect();
            }
        }

        Ok(Value::Null)
    }
);

native_fn_with_data!(
    TextInputClearMethod,
    "clear",
    0,
    TextInputData,
    |_evaluator, _args, _cursor, data| {
        let mut d = data.borrow_mut();
        d.content.clear();
        d.cursor = 0;
        Ok(Value::Null)
    }
);

native_fn_with_data!(
    TextInputSetFocusedMethod,
    "set_focused",
    1,
    TextInputData,
    |_evaluator, args, _cursor, data| {
        let focused = match &args[0] {
            Value::Bool(b) => *b,
            _ => return Ok(Value::Null),
        };

        data.borrow_mut().focused = focused;
        Ok(Value::Null)
    }
);

native_fn_with_data!(
    TextInputSetStyleMethod,
    "set_style",
    3,
    TextInputData,
    |_evaluator, args, _cursor, data| {
        let style = TuiStyle::from_args(Some(&args[0]), Some(&args[1]), Some(&args[2]));

        data.borrow_mut().style = style;

        Ok(Value::Null)
    }
);

native_fn_with_data!(
    TextInputRenderMethod,
    "render",
    0,
    TextInputData,
    |_evaluator, _args, _cursor, data| {
        let d = data.borrow();

        WIDGETS.with(|w| {
            w.borrow_mut().push(Widget::TextInput(TextInputWidget {
                x: d.x,
                y: d.y,
                width: d.width,
                content: d.content.clone(),
                cursor: d.cursor,
                placeholder: d.placeholder.clone(),
                focused: d.focused,
                style: d.style.clone(),
            }));
        });

        Ok(Value::Null)
    }
);

#[derive(Clone)]
pub struct TextInputWidget {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub content: String,
    pub cursor: usize,
    pub placeholder: String,
    pub focused: bool,
    pub style: TuiStyle,
}

pub fn render_text_input(frame: &mut Frame<'_>, widget: &TextInputWidget, area: Rect) {
    let display_text = if widget.content.is_empty() {
        if widget.focused {
            String::new()
        } else {
            widget.placeholder.clone()
        }
    } else {
        widget.content.clone()
    };

    let inner_width = widget.width.saturating_sub(2) as usize;
    let chars: Vec<char> = display_text.chars().collect();
    let scroll_offset = if widget.cursor > inner_width {
        widget.cursor.saturating_sub(inner_width)
    } else {
        0
    };
    let visible_end = (scroll_offset + inner_width).min(chars.len());
    let visible_text: String = chars[scroll_offset..visible_end].iter().collect();

    let display_with_cursor = if widget.focused {
        let cursor_pos = widget.cursor.saturating_sub(scroll_offset);
        let mut chars: Vec<char> = visible_text.chars().collect();
        if cursor_pos <= chars.len() {
            chars.insert(cursor_pos, '│');
        }
        chars.iter().collect()
    } else {
        visible_text
    };

    let paragraph = Paragraph::new(display_with_cursor)
        .style(widget.style.text_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(widget.style.border_style(widget.focused)),
        );

    frame.render_widget(paragraph, area);
}
