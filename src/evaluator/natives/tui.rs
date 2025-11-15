mod canvas;
mod text_input;

use std::{cell::RefCell, collections::HashMap, io, rc::Rc};

use crate::{
    evaluator::{
        Callable, EvalResult, Evaluator,
        natives::tui::{
            canvas::{CanvasWidget, FnTuiCreateCanvas, render_canvas},
            text_input::{FnTuiCreateTextInput, TextInputWidget, render_text_input},
        },
        object::{Method, NativeMethod, Object},
        value::Value,
    },
    native_fn,
};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
};

pub fn native_tui() -> Value {
    let mut methods: HashMap<String, Method> = HashMap::new();

    methods.insert(
        "init".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTuiInit), false)),
    );
    methods.insert(
        "cleanup".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTuiCleanup), false)),
    );
    methods.insert(
        "draw_block".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTuiDrawBlock), false)),
    );
    methods.insert(
        "draw_text".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTuiDrawText), false)),
    );
    methods.insert(
        "draw_list".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTuiDrawList), false)),
    );
    methods.insert(
        "draw_progress".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTuiDrawProgress), false)),
    );
    methods.insert(
        "clear".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTuiClear), false)),
    );
    methods.insert(
        "render".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTuiRender), false)),
    );

    methods.insert(
        "create_canvas".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTuiCreateCanvas), false)),
    );
    methods.insert(
        "create_text_input".into(),
        Method::Native(NativeMethod::new(Rc::new(FnTuiCreateTextInput), false)),
    );

    Value::Obj(Rc::new(Object::new("Tui".into(), methods)))
}

// Widget types to accumulate before rendering
#[derive(Clone)]
enum Widget {
    Block {
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        title: String,
        style: TuiStyle,
    },
    Text {
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        text: String,
        style: TuiStyle,
    },
    List {
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        items: Vec<String>,
        selected: usize,
        style: TuiStyle,
        title: String,
    },
    Progress {
        x: u16,
        y: u16,
        width: u16,
        percent: u16,
        label: String,
        style: TuiStyle,
    },
    Canvas(CanvasWidget),
    TextInput(TextInputWidget),
}

impl Widget {
    fn render(&self, frame: &mut Frame<'_>) {
        match self {
            Widget::Block {
                x,
                y,
                width,
                height,
                title,
                style,
            } => {
                let area = Rect::new(*x, *y, *width, *height);
                let block = Block::default()
                    .title(title.clone())
                    .borders(Borders::ALL)
                    .style(style.text_style())
                    .border_style(Style::default().fg(style.accent));
                frame.render_widget(block, area);
            }
            Widget::Text {
                x,
                y,
                width,
                height,
                text,
                style,
            } => {
                let area = Rect::new(*x, *y, *width, *height);
                let paragraph = Paragraph::new(text.clone())
                    .style(style.text_style())
                    .wrap(Wrap { trim: false });
                frame.render_widget(paragraph, area);
            }
            Widget::List {
                x,
                y,
                width,
                height,
                items,
                selected,
                style,
                title,
            } => {
                let area = Rect::new(*x, *y, *width, *height);
                let normal = style.text_style();
                let highlight = Style::default()
                    .fg(style.accent)
                    .bg(style.bg)
                    .add_modifier(Modifier::BOLD);

                let list_items: Vec<ListItem> = items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let prefix = if i == *selected { "> " } else { "  " };
                        let item_style = if i == *selected { highlight } else { normal };
                        ListItem::new(format!("{}{}", prefix, item)).style(item_style)
                    })
                    .collect();

                let list = List::new(list_items).block(
                    Block::default()
                        .title(title.clone())
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(style.accent)),
                );

                frame.render_widget(list, area);
            }
            Widget::Progress {
                x,
                y,
                width,
                percent,
                label,
                style,
            } => {
                let area = Rect::new(*x, *y, *width, 3);
                let gauge = Gauge::default()
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(style.accent)),
                    )
                    .gauge_style(style.text_style().fg(style.accent))
                    .percent(*percent)
                    .label(label.clone());
                frame.render_widget(gauge, area);
            }
            Widget::Canvas(widget) => render_canvas(frame, widget, widget_rect(frame, widget.x, widget.y, widget.width, widget.height)),
            Widget::TextInput(widget) => render_text_input(frame, widget, widget_rect(frame, widget.x, widget.y, widget.width, 3)),
        }
    }
}

pub(super) fn widget_rect(frame: &Frame<'_>, x: u16, y: u16, width: u16, height: u16) -> Rect {
    let parent = frame.area();
    let y = y.min(parent.height);
    let x = x.min(parent.width);
    let height = height.min(parent.height.saturating_sub(y));
    let width = width.min(parent.width.saturating_sub(x));

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(y),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(parent);
    let row_area = rows[1];

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(x),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(row_area)[1]
}

#[derive(Clone)]
pub struct TuiStyle {
    pub fg: Color,
    pub bg: Color,
    pub accent: Color,
}

impl Default for TuiStyle {
    fn default() -> Self {
        Self {
            fg: Color::White,
            bg: Color::Reset,
            accent: Color::Cyan,
        }
    }
}

impl TuiStyle {
    fn color_from_value(val: Option<&Value>, default: Color) -> Color {
        match val {
            Some(Value::Str(s)) => parse_color(&s.borrow()),
            Some(Value::Null) => Color::Reset,
            _ => default,
        }
    }

    fn with_fg(mut self, fg: Color) -> Self {
        self.fg = fg;
        self
    }

    fn with_bg(mut self, bg: Color) -> Self {
        self.bg = bg;
        self
    }

    fn with_accent(mut self, accent: Color) -> Self {
        self.accent = accent;
        self
    }

    fn from_args(
        fg_arg: Option<&Value>,
        bg_arg: Option<&Value>,
        accent_arg: Option<&Value>,
    ) -> Self {
        Self::default()
            .with_fg(Self::color_from_value(fg_arg, Color::White))
            .with_bg(Self::color_from_value(bg_arg, Color::Reset))
            .with_accent(Self::color_from_value(accent_arg, Color::Cyan))
    }

    fn text_style(&self) -> Style {
        Style::default().fg(self.fg).bg(self.bg)
    }

    fn accent_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    fn border_style(&self, focused: bool) -> Style {
        let base = self.accent_style();
        if focused {
            base.add_modifier(Modifier::BOLD)
        } else {
            base
        }
    }
}

// Global terminal instance and widget buffer
thread_local! {
    static TERMINAL: RefCell<Option<Terminal<CrosstermBackend<io::Stdout>>>> = RefCell::new(None);
    static WIDGETS: RefCell<Vec<Widget>> = RefCell::new(Vec::new());
}

// Tui.init(): initializes the TUI (enters alternate screen, raw mode)
native_fn!(FnTuiInit, "tui_init", 0, |_evaluator, _args, _cursor| {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    TERMINAL.with(|t| {
        *t.borrow_mut() = Some(terminal);
    });

    Ok(Value::Null)
});

// Tui.cleanup(): cleans up the TUI (exits alternate screen, restores terminal)
native_fn!(
    FnTuiCleanup,
    "tui_cleanup",
    0,
    |_evaluator, _args, _cursor| {
        TERMINAL.with(|t| {
            if let Some(mut terminal) = t.borrow_mut().take() {
                let _ = disable_raw_mode();
                let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
                let _ = terminal.show_cursor();
            }
        });

        Ok(Value::Null)
    }
);

// Tui.clear(): clears the widget buffer (call this at the start of each frame)
native_fn!(FnTuiClear, "tui_clear", 0, |_evaluator, _args, _cursor| {
    WIDGETS.with(|w| {
        w.borrow_mut().clear();
    });

    Ok(Value::Null)
});

// Tui.render(): renders all accumulated widgets to the screen
native_fn!(
    FnTuiRender,
    "tui_render",
    0,
    |_evaluator, _args, _cursor| {
        let result = TERMINAL.with(|t| -> io::Result<()> {
            if let Some(terminal) = t.borrow_mut().as_mut() {
                terminal.draw(|frame| {
                    WIDGETS.with(|w| {
                        for widget in w.borrow().iter() {
                            widget.render(frame);
                        }
                    });
                })?;
            }
            Ok(())
        });

        result?;
        Ok(Value::Null)
    }
);

// Tui.draw_block(x, y, width, height, title, border_color)
native_fn!(
    FnTuiDrawBlock,
    "tui_draw_block",
    6,
    |_evaluator, args, cursor| {
        let x = args[0].check_num(cursor, Some("x position".into()))? as u16;
        let y = args[1].check_num(cursor, Some("y position".into()))? as u16;
        let width = args[2].check_num(cursor, Some("width".into()))? as u16;
        let height = args[3].check_num(cursor, Some("height".into()))? as u16;

        let title = string_from_value(&args[4]);
        let style = TuiStyle::from_args(None, None, args.get(5));

        WIDGETS.with(|w| {
            w.borrow_mut().push(Widget::Block {
                x,
                y,
                width,
                height,
                title,
                style,
            });
        });

        Ok(Value::Null)
    }
);

// Tui.draw_text(x, y, width, height, text, fg_color, bg_color)
native_fn!(
    FnTuiDrawText,
    "tui_draw_text",
    7,
    |_evaluator, args, cursor| {
        let x = args[0].check_num(cursor, Some("x position".into()))? as u16;
        let y = args[1].check_num(cursor, Some("y position".into()))? as u16;
        let width = args[2].check_num(cursor, Some("width".into()))? as u16;
        let height = args[3].check_num(cursor, Some("height".into()))? as u16;

        let text = string_from_value(&args[4]);
        let style = TuiStyle::from_args(args.get(5), args.get(6), None);

        WIDGETS.with(|w| {
            w.borrow_mut().push(Widget::Text {
                x,
                y,
                width,
                height,
                text,
                style,
            });
        });

        Ok(Value::Null)
    }
);

// Tui.draw_list(x, y, width, height, items, selected, color, title)
// items: List of strings, selected: index of selected item
native_fn!(
    FnTuiDrawList,
    "tui_draw_list",
    8,
    |_evaluator, args, cursor| {
        let x = args[0].check_num(cursor, Some("x".into()))? as u16;
        let y = args[1].check_num(cursor, Some("y".into()))? as u16;
        let width = args[2].check_num(cursor, Some("width".into()))? as u16;
        let height = args[3].check_num(cursor, Some("height".into()))? as u16;

        let items = match &args[4] {
            Value::List(list) => list
                .borrow()
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
            _ => vec![],
        };

        let selected_val = args[5].check_num(cursor, Some("selected index".into()))?;
        let selected = if selected_val < 0.0 {
            0
        } else {
            selected_val as usize
        };

        let style = TuiStyle::from_args(None, None, args.get(6));
        let title = string_from_value(&args[7]);

        WIDGETS.with(|w| {
            w.borrow_mut().push(Widget::List {
                x,
                y,
                width,
                height,
                items,
                selected,
                style,
                title,
            });
        });

        Ok(Value::Null)
    }
);

// Tui.draw_progress(x, y, width, percent, label, color)
// percent: 0-100
native_fn!(
    FnTuiDrawProgress,
    "tui_draw_progress",
    6,
    |_evaluator, args, cursor| {
        let x = args[0].check_num(cursor, Some("x".into()))? as u16;
        let y = args[1].check_num(cursor, Some("y".into()))? as u16;
        let width = args[2].check_num(cursor, Some("width".into()))? as u16;
        let percent = args[3]
            .check_num(cursor, Some("percent".into()))?
            .clamp(0.0, 100.0) as u16;

        let label = string_from_value(&args[4]);
        let style = TuiStyle::from_args(None, None, args.get(5));

        WIDGETS.with(|w| {
            w.borrow_mut().push(Widget::Progress {
                x,
                y,
                width,
                percent,
                label,
                style,
            });
        });

        Ok(Value::Null)
    }
);

// Helper function to parse color strings
pub fn parse_color(s: &str) -> Color {
    match s.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "darkgrey" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        _ => Color::White,
    }
}

fn string_from_value(value: &Value) -> String {
    match value {
        Value::Str(s) => s.borrow().clone(),
        _ => String::new(),
    }
}
