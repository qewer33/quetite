use crate::{
    evaluator::natives::tui::{WIDGETS, Widget, parse_color},
    native_fn, native_fn_with_data,
};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::evaluator::{
    Callable, EvalResult, Evaluator,
    object::{Method, NativeMethod, Object},
    value::Value,
};

use ratatui::{
    Frame,
    layout::Rect,
    style::Color,
    widgets::canvas::{Canvas as RatatuiCanvas, Circle, Line, Points, Rectangle},
};

// Tui.create_canvas(x, y, width, height) -> Canvas object
native_fn!(
    FnTuiCreateCanvas,
    "tui_create_canvas",
    4,
    |_evaluator, args, cursor| {
        let x = args[0].check_num(cursor, Some("x position".into()))? as u16;
        let y = args[1].check_num(cursor, Some("y position".into()))? as u16;
        let width = args[2].check_num(cursor, Some("width".into()))? as u16;
        let height = args[3].check_num(cursor, Some("height".into()))? as u16;

        let canvas_data = Rc::new(RefCell::new(CanvasData {
            x,
            y,
            width,
            height,
            x_bounds: (0.0, 100.0),
            y_bounds: (0.0, 100.0),
            commands: Vec::new(),
        }));

        let mut methods: HashMap<String, Method> = HashMap::new();

        methods.insert(
            "line".into(),
            Method::Native(NativeMethod::new(
                Rc::new(CanvasLineMethod {
                    data: Rc::clone(&canvas_data),
                }),
                false,
            )),
        );

        methods.insert(
            "circle".into(),
            Method::Native(NativeMethod::new(
                Rc::new(CanvasCircleMethod {
                    data: Rc::clone(&canvas_data),
                }),
                false,
            )),
        );

        methods.insert(
            "rectangle".into(),
            Method::Native(NativeMethod::new(
                Rc::new(CanvasRectangleMethod {
                    data: Rc::clone(&canvas_data),
                }),
                false,
            )),
        );

        methods.insert(
            "points".into(),
            Method::Native(NativeMethod::new(
                Rc::new(CanvasPointsMethod {
                    data: Rc::clone(&canvas_data),
                }),
                false,
            )),
        );

        methods.insert(
            "set_bounds".into(),
            Method::Native(NativeMethod::new(
                Rc::new(CanvasSetBoundsMethod {
                    data: Rc::clone(&canvas_data),
                }),
                false,
            )),
        );

        methods.insert(
            "clear".into(),
            Method::Native(NativeMethod::new(
                Rc::new(CanvasClearMethod {
                    data: Rc::clone(&canvas_data),
                }),
                false,
            )),
        );

        methods.insert(
            "render".into(),
            Method::Native(NativeMethod::new(
                Rc::new(CanvasRenderMethod {
                    data: Rc::clone(&canvas_data),
                }),
                false,
            )),
        );

        Ok(Value::Obj(Rc::new(Object::new("Canvas".into(), methods))))
    }
);

pub struct CanvasData {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    x_bounds: (f64, f64),
    y_bounds: (f64, f64),
    commands: Vec<CanvasCommand>,
}

#[derive(Clone)]
pub enum CanvasCommand {
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: Color,
    },
    Circle {
        x: f64,
        y: f64,
        radius: f64,
        color: Color,
    },
    Rectangle {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color: Color,
    },
    Points {
        points: Vec<(f64, f64)>,
        color: Color,
    },
}

#[derive(Clone)]
pub struct CanvasWidget {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub x_bounds: (f64, f64),
    pub y_bounds: (f64, f64),
    pub commands: Vec<CanvasCommand>,
}

pub fn render_canvas(frame: &mut Frame<'_>, widget: &CanvasWidget, area: Rect) {
    let canvas = RatatuiCanvas::default()
        .x_bounds([widget.x_bounds.0, widget.x_bounds.1])
        .y_bounds([widget.y_bounds.0, widget.y_bounds.1])
        .paint(|ctx| {
            for cmd in &widget.commands {
                match cmd {
                    CanvasCommand::Line {
                        x1,
                        y1,
                        x2,
                        y2,
                        color,
                    } => ctx.draw(&Line {
                        x1: *x1,
                        y1: *y1,
                        x2: *x2,
                        y2: *y2,
                        color: *color,
                    }),
                    CanvasCommand::Circle {
                        x,
                        y,
                        radius,
                        color,
                    } => ctx.draw(&Circle {
                        x: *x,
                        y: *y,
                        radius: *radius,
                        color: *color,
                    }),
                    CanvasCommand::Rectangle {
                        x,
                        y,
                        width,
                        height,
                        color,
                    } => ctx.draw(&Rectangle {
                        x: *x,
                        y: *y,
                        width: *width,
                        height: *height,
                        color: *color,
                    }),
                    CanvasCommand::Points { points, color } => ctx.draw(&Points {
                        coords: points,
                        color: *color,
                    }),
                }
            }
        });
    frame.render_widget(canvas, area);
}

// Canvas method implementations using the macro

native_fn_with_data!(
    CanvasLineMethod,
    "line",
    5,
    CanvasData,
    |_evaluator, args, cursor, data| {
        let x1 = args[0].check_num(cursor, Some("x1".into()))?;
        let y1 = args[1].check_num(cursor, Some("y1".into()))?;
        let x2 = args[2].check_num(cursor, Some("x2".into()))?;
        let y2 = args[3].check_num(cursor, Some("y2".into()))?;
        let color = args
            .get(4)
            .and_then(|v| match v {
                Value::Str(s) => Some(parse_color(&s.borrow())),
                _ => None,
            })
            .unwrap_or(Color::White);

        data.borrow_mut().commands.push(CanvasCommand::Line {
            x1,
            y1,
            x2,
            y2,
            color,
        });

        Ok(Value::Null)
    }
);

native_fn_with_data!(
    CanvasCircleMethod,
    "circle",
    4,
    CanvasData,
    |_evaluator, args, cursor, data| {
        let x = args[0].check_num(cursor, Some("x".into()))?;
        let y = args[1].check_num(cursor, Some("y".into()))?;
        let radius = args[2].check_num(cursor, Some("radius".into()))?;
        let color = args
            .get(3)
            .and_then(|v| match v {
                Value::Str(s) => Some(parse_color(&s.borrow())),
                _ => None,
            })
            .unwrap_or(Color::White);

        data.borrow_mut().commands.push(CanvasCommand::Circle {
            x,
            y,
            radius,
            color,
        });

        Ok(Value::Null)
    }
);

native_fn_with_data!(
    CanvasRectangleMethod,
    "rectangle",
    5,
    CanvasData,
    |_evaluator, args, cursor, data| {
        let x = args[0].check_num(cursor, Some("x".into()))?;
        let y = args[1].check_num(cursor, Some("y".into()))?;
        let width = args[2].check_num(cursor, Some("width".into()))?;
        let height = args[3].check_num(cursor, Some("height".into()))?;
        let color = args
            .get(4)
            .and_then(|v| match v {
                Value::Str(s) => Some(parse_color(&s.borrow())),
                _ => None,
            })
            .unwrap_or(Color::White);

        data.borrow_mut().commands.push(CanvasCommand::Rectangle {
            x,
            y,
            width,
            height,
            color,
        });

        Ok(Value::Null)
    }
);

native_fn_with_data!(
    CanvasPointsMethod,
    "points",
    2,
    CanvasData,
    |_evaluator, args, _cursor, data| {
        let points = match &args[0] {
            Value::List(list) => {
                let borrowed = list.borrow();
                let mut coords = Vec::new();
                for value in borrowed.iter() {
                    if let Value::List(pair) = value {
                        let pair_ref = pair.borrow();
                        if pair_ref.len() != 2 {
                            return Ok(Value::Null);
                        }
                        let x = match &pair_ref[0] {
                            Value::Num(n) => n.0,
                            _ => return Ok(Value::Null),
                        };
                        let y = match &pair_ref[1] {
                            Value::Num(n) => n.0,
                            _ => return Ok(Value::Null),
                        };
                        coords.push((x, y));
                    } else {
                        return Ok(Value::Null);
                    }
                }
                coords
            }
            _ => return Ok(Value::Null),
        };

        let color = args
            .get(1)
            .and_then(|v| match v {
                Value::Str(s) => Some(parse_color(&s.borrow())),
                _ => None,
            })
            .unwrap_or(Color::White);

        data.borrow_mut()
            .commands
            .push(CanvasCommand::Points { points, color });

        Ok(Value::Null)
    }
);

native_fn_with_data!(
    CanvasSetBoundsMethod,
    "set_bounds",
    4,
    CanvasData,
    |_evaluator, args, cursor, data| {
        let x_min = args[0].check_num(cursor, Some("min x".into()))?;
        let x_max = args[1].check_num(cursor, Some("max x".into()))?;
        let y_min = args[2].check_num(cursor, Some("min y".into()))?;
        let y_max = args[3].check_num(cursor, Some("max y".into()))?;

        let mut d = data.borrow_mut();
        d.x_bounds = (x_min, x_max);
        d.y_bounds = (y_min, y_max);

        Ok(Value::Null)
    }
);

native_fn_with_data!(
    CanvasClearMethod,
    "clear",
    0,
    CanvasData,
    |_evaluator, _args, _cursor, data| {
        data.borrow_mut().commands.clear();
        Ok(Value::Null)
    }
);

native_fn_with_data!(
    CanvasRenderMethod,
    "render",
    0,
    CanvasData,
    |_evaluator, _args, _cursor, data| {
        let d = data.borrow();

        WIDGETS.with(|w| {
            w.borrow_mut().push(Widget::Canvas(CanvasWidget {
                x: d.x,
                y: d.y,
                width: d.width,
                height: d.height,
                x_bounds: d.x_bounds,
                y_bounds: d.y_bounds,
                commands: d.commands.clone(),
            }));
        });

        Ok(Value::Null)
    }
);
