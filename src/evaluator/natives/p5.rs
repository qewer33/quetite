use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use once_cell::sync::Lazy;
use pixels::{Pixels, SurfaceTexture};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};
#[cfg(target_os = "linux")]
use winit::platform::{wayland::EventLoopBuilderExtWayland, x11::EventLoopBuilderExtX11};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
#[cfg(not(target_os = "linux"))]
use winit::event_loop::EventLoop;

use crate::{
    evaluator::{
        Callable, Evaluator,
        object::{Method, NativeMethod, Object},
        runtime_err::{EvalResult, RuntimeEvent},
        value::Value,
    },
    lexer::cursor::Cursor,
    native_fn,
};

static P5_RUNTIME: Lazy<Mutex<Option<P5Runtime>>> = Lazy::new(|| Mutex::new(None));

type SharedState = Arc<Mutex<P5State>>;

thread_local! {
    static P5_CALLBACKS: RefCell<P5Callbacks> = RefCell::new(P5Callbacks::default());
}

#[derive(Clone, Default)]
struct P5Callbacks {
    setup: Option<Rc<dyn Callable>>,
    draw: Option<Rc<dyn Callable>>,
}

#[derive(Clone)]
struct P5Runtime {
    state: SharedState,
    cmd_tx: mpsc::Sender<P5Command>,
}

impl P5Runtime {
    fn new(state: SharedState, cmd_tx: mpsc::Sender<P5Command>) -> Self {
        Self { state, cmd_tx }
    }

    fn state(&self) -> SharedState {
        Arc::clone(&self.state)
    }

    fn send(&self, cmd: P5Command) {
        let _ = self.cmd_tx.send(cmd);
    }

    fn begin_frame(&self) -> FrameGuard {
        FrameGuard::new(&self.state)
    }
}

#[derive(Debug)]
enum P5Command {
    Resize(u32, u32),
}

struct FrameGuard {
    state: SharedState,
}

impl FrameGuard {
    fn new(state: &SharedState) -> Self {
        {
            let mut lock = state.lock().unwrap();
            lock.frame_in_progress = true;
        }
        Self {
            state: Arc::clone(state),
        }
    }
}

impl Drop for FrameGuard {
    fn drop(&mut self) {
        let mut lock = self.state.lock().unwrap();
        lock.frame_in_progress = false;
    }
}

pub fn native_p5() -> Value {
    let mut methods: HashMap<String, Method> = HashMap::new();

    methods.insert(
        "rect".into(),
        Method::Native(NativeMethod::new(Rc::new(FnP5Rect), false)),
    );
    methods.insert(
        "circle".into(),
        Method::Native(NativeMethod::new(Rc::new(FnP5Circle), false)),
    );
    methods.insert(
        "ellipse".into(),
        Method::Native(NativeMethod::new(Rc::new(FnP5Ellipse), false)),
    );
    methods.insert(
        "line".into(),
        Method::Native(NativeMethod::new(Rc::new(FnP5Line), false)),
    );
    methods.insert(
        "background".into(),
        Method::Native(NativeMethod::new(Rc::new(FnP5Background), false)),
    );
    methods.insert(
        "fill".into(),
        Method::Native(NativeMethod::new(Rc::new(FnP5Fill), false)),
    );
    methods.insert(
        "stroke".into(),
        Method::Native(NativeMethod::new(Rc::new(FnP5Stroke), false)),
    );
    methods.insert(
        "no_fill".into(),
        Method::Native(NativeMethod::new(Rc::new(FnP5NoFill), false)),
    );
    methods.insert(
        "no_stroke".into(),
        Method::Native(NativeMethod::new(Rc::new(FnP5NoStroke), false)),
    );
    methods.insert(
        "stroke_weight".into(),
        Method::Native(NativeMethod::new(Rc::new(FnP5StrokeWeight), false)),
    );
    methods.insert(
        "setup".into(),
        Method::Native(NativeMethod::new(Rc::new(FnP5Setup), false)),
    );
    methods.insert(
        "draw".into(),
        Method::Native(NativeMethod::new(Rc::new(FnP5Draw), false)),
    );
    methods.insert(
        "size".into(),
        Method::Native(NativeMethod::new(Rc::new(FnP5Size), false)),
    );
    methods.insert(
        "run".into(),
        Method::Native(NativeMethod::new(Rc::new(FnP5Run), false)),
    );

    Value::Obj(Rc::new(Object::new("P5".into(), methods)))
}

const DEFAULT_WIDTH: usize = 640;
const DEFAULT_HEIGHT: usize = 480;

#[derive(Debug)]
struct P5State {
    width: usize,
    height: usize,
    buffer: Vec<u8>,
    dirty: bool,
    open: bool,
    frame_in_progress: bool,
    fill_color: Option<Color>,
    stroke_color: Option<Color>,
    stroke_weight: f32,
}

impl P5State {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; width * height * 4],
            dirty: true,
            open: true,
            fill_color: Some(Color::from_rgba8(255, 255, 255, 255)),
            stroke_color: Some(Color::from_rgba8(255, 255, 255, 255)),
            frame_in_progress: false,
            stroke_weight: 1.0,
        }
    }

    fn pixmap_mut(&mut self) -> PixmapMut<'_> {
        PixmapMut::from_bytes(&mut self.buffer, self.width as u32, self.height as u32)
            .expect("invalid pixmap size")
    }

    fn background(&mut self, color: Color) {
        self.pixmap_mut().fill(color);
        self.dirty = true;
    }

    fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        if let Some(rect) = Rect::from_xywh(x, y, w, h) {
            let fill = self.fill_color;
            let stroke = self.stroke_color;
            let stroke_width = self.stroke_weight;
            let mut pixmap = self.pixmap_mut();
            if let Some(color) = fill {
                let mut paint = Paint::default();
                paint.set_color(color);
                pixmap.fill_rect(rect, &paint, Transform::identity(), None);
            }
            if let Some(color) = stroke {
                let mut paint = Paint::default();
                paint.set_color(color);
                let mut stroke = Stroke::default();
                stroke.width = stroke_width.max(0.1);
                let mut pb = PathBuilder::new();
                pb.push_rect(rect);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
            self.dirty = true;
        }
    }

    fn draw_oval(&mut self, cx: f32, cy: f32, rx: f32, ry: f32) {
        if rx <= 0.0 || ry <= 0.0 {
            return;
        }
        if let Some(rect) = Rect::from_xywh(cx - rx, cy - ry, rx * 2.0, ry * 2.0) {
            let fill = self.fill_color;
            let stroke = self.stroke_color;
            let stroke_width = self.stroke_weight;
            let mut pixmap = self.pixmap_mut();
            if let Some(color) = fill {
                if let Some(path) = PathBuilder::from_oval(rect) {
                    let mut paint = Paint::default();
                    paint.set_color(color);
                    pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
                }
            }
            if let Some(color) = stroke {
                if let Some(path) = PathBuilder::from_oval(rect) {
                    let mut paint = Paint::default();
                    paint.set_color(color);
                    let mut stroke = Stroke::default();
                    stroke.width = stroke_width.max(0.1);
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
            self.dirty = true;
        }
    }

    fn draw_circle(&mut self, cx: f32, cy: f32, diameter: f32) {
        self.draw_oval(cx, cy, diameter / 2.0, diameter / 2.0);
    }

    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let stroke_color = match self.stroke_color {
            Some(color) => color,
            None => return,
        };
        let stroke_width = self.stroke_weight;
        let mut pb = PathBuilder::new();
        pb.move_to(x1, y1);
        pb.line_to(x2, y2);
        if let Some(path) = pb.finish() {
            let mut pixmap = self.pixmap_mut();
            let mut paint = Paint::default();
            paint.set_color(stroke_color);
            let mut stroke = Stroke::default();
            stroke.width = stroke_width.max(0.1);
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            self.dirty = true;
        }
    }
}
fn cleanup_runtime() {
    let mut runtime = P5_RUNTIME.lock().unwrap();
    if let Some(rt) = runtime.as_ref() {
        if !rt.state.lock().unwrap().open {
            *runtime = None;
        }
    }
}

fn current_runtime() -> Option<P5Runtime> {
    cleanup_runtime();
    P5_RUNTIME.lock().unwrap().clone()
}

fn set_runtime(runtime: P5Runtime) {
    let mut slot = P5_RUNTIME.lock().unwrap();
    *slot = Some(runtime);
}

fn ensure_runtime(cursor: Cursor) -> EvalResult<P5Runtime> {
    if let Some(handles) = current_runtime() {
        return Ok(handles);
    }
    let handles = start_window_thread(DEFAULT_WIDTH, DEFAULT_HEIGHT)
        .map_err(|msg| RuntimeEvent::error(format!("failed to create P5 window: {msg}"), cursor))?;
    set_runtime(handles.clone());
    Ok(handles)
}

fn get_runtime(cursor: Cursor) -> EvalResult<P5Runtime> {
    current_runtime()
        .ok_or_else(|| RuntimeEvent::error("call P5.run() before using P5 methods".into(), cursor))
}

fn start_window_thread(width: usize, height: usize) -> Result<P5Runtime, String> {
    let state = Arc::new(Mutex::new(P5State::new(width, height)));
    let (cmd_tx, cmd_rx) = mpsc::channel::<P5Command>();
    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();
    let state_for_thread = Arc::clone(&state);

    thread::Builder::new()
        .name("p5-window".into())
        .spawn(move || {
            let event_loop = {
                #[cfg(target_os = "linux")]
                {
                    let mut builder = EventLoopBuilder::<()>::new();
                    EventLoopBuilderExtWayland::with_any_thread(&mut builder, true);
                    EventLoopBuilderExtX11::with_any_thread(&mut builder, true);
                    builder.build()
                }
                #[cfg(not(target_os = "linux"))]
                {
                    EventLoop::new()
                }
            };
            let window = match WindowBuilder::new()
                .with_title("P5 Window")
                .with_inner_size(LogicalSize::new(width as f64, height as f64))
                .with_resizable(false)
                .build(&event_loop)
            {
                Ok(win) => win,
                Err(err) => {
                    let _ = ready_tx.send(Err(err.to_string()));
                    return;
                }
            };

            let surface = {
                let size = window.inner_size();
                SurfaceTexture::new(size.width, size.height, &window)
            };

            let mut pixels = match Pixels::new(width as u32, height as u32, surface) {
                Ok(p) => p,
                Err(err) => {
                    let _ = ready_tx.send(Err(err.to_string()));
                    return;
                }
            };

            let _ = ready_tx.send(Ok(()));

            event_loop.run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Poll;
                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => {
                            state_for_thread.lock().unwrap().open = false;
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::Resized(size) => {
                            if let Err(err) = pixels.resize_surface(size.width, size.height) {
                                eprintln!("P5 resize error: {err}");
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            if let Err(err) =
                                pixels.resize_surface(new_inner_size.width, new_inner_size.height)
                            {
                                eprintln!("P5 scale error: {err}");
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                        _ => {}
                    },
                    Event::RedrawRequested(_) => {
                        if !render_frame(&mut pixels, &state_for_thread) {
                            state_for_thread.lock().unwrap().open = false;
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    Event::MainEventsCleared => {
                        while let Ok(cmd) = cmd_rx.try_recv() {
                            match cmd {
                                P5Command::Resize(w, h) => {
                                    window.set_inner_size(LogicalSize::new(w as f64, h as f64));
                                    if let Err(err) = pixels.resize_buffer(w, h) {
                                        eprintln!("P5 buffer resize error: {err}");
                                    }
                                    if let Err(err) = pixels.resize_surface(w, h) {
                                        eprintln!("P5 surface resize error: {err}");
                                    }
                                }
                            }
                        }

                        let keep_running = { state_for_thread.lock().unwrap().open };
                        if !keep_running {
                            *control_flow = ControlFlow::Exit;
                        } else {
                            window.request_redraw();
                        }
                    }
                    _ => {}
                }
            });
        })
        .map_err(|err| err.to_string())?;

    match ready_rx.recv() {
        Ok(Ok(())) => Ok(P5Runtime::new(state, cmd_tx)),
        Ok(Err(msg)) => Err(msg),
        Err(_) => Err("failed to initialize P5 window".into()),
    }
}

fn render_frame(pixels: &mut Pixels, state: &SharedState) -> bool {
    let mut should_render = false;
    {
        let mut lock = state.lock().unwrap();
        if lock.frame_in_progress {
            return true;
        }
        if lock.dirty {
            let frame = pixels.frame_mut();
            if frame.len() == lock.buffer.len() {
                frame.copy_from_slice(&lock.buffer);
            }
            lock.dirty = false;
            should_render = true;
        }
    }
    if should_render {
        pixels.render().is_ok()
    } else {
        true
    }
}

fn convert_len(value: f64, name: &str, cursor: Cursor) -> EvalResult<usize> {
    if value <= 0.0 {
        return Err(RuntimeEvent::error(
            format!("{name} must be greater than zero"),
            cursor,
        ));
    }
    if (value.fract()).abs() > f64::EPSILON {
        return Err(RuntimeEvent::error(
            format!("{name} must be an integer"),
            cursor,
        ));
    }
    Ok(value as usize)
}

fn clamp_to_usize(value: f64) -> usize {
    if value.is_sign_negative() {
        0
    } else {
        value.floor() as usize
    }
}

fn color_from_rgb(r: f64, g: f64, b: f64) -> Color {
    Color::from_rgba8(
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
        255,
    )
}

fn lookup_env_callable(
    evaluator: &mut Evaluator,
    name: &str,
    cursor: Cursor,
) -> EvalResult<Option<Rc<dyn Callable>>> {
    let result = evaluator.env.borrow().get(name, cursor);
    match result {
        Ok(Value::Callable(cb)) => Ok(Some(cb)),
        Ok(_) => Err(RuntimeEvent::error(
            format!("function '{name}' must be callable"),
            cursor,
        )),
        Err(_) => Ok(None),
    }
}

fn ensure_callable(value: &Value, cursor: Cursor, label: &str) -> EvalResult<Rc<dyn Callable>> {
    if let Value::Callable(cb) = value {
        Ok(Rc::clone(cb))
    } else {
        Err(RuntimeEvent::error(
            format!("{label} must be a function"),
            cursor,
        ))
    }
}

native_fn!(FnP5Rect, "p5_rect", 4, |_evaluator, args, cursor| {
    let x = clamp_to_usize(args[0].check_num(cursor, Some("x".into()))?);
    let y = clamp_to_usize(args[1].check_num(cursor, Some("y".into()))?);
    let w = clamp_to_usize(args[2].check_num(cursor, Some("width".into()))?);
    let h = clamp_to_usize(args[3].check_num(cursor, Some("height".into()))?);

    if w == 0 || h == 0 {
        return Ok(Value::Null);
    }

    let runtime = get_runtime(cursor)?;
    {
        let state = runtime.state();
        let mut lock = state.lock().unwrap();
        if !lock.open {
            return Err(RuntimeEvent::error(
                "P5 window is closed; call P5.run() first".into(),
                cursor,
            ));
        }
        let clamped_x = x.min(lock.width);
        let clamped_y = y.min(lock.height);
        let clamped_w = w.min(lock.width.saturating_sub(clamped_x));
        let clamped_h = h.min(lock.height.saturating_sub(clamped_y));
        if clamped_w == 0 || clamped_h == 0 {
            return Ok(Value::Null);
        }

        lock.draw_rect(
            clamped_x as f32,
            clamped_y as f32,
            clamped_w as f32,
            clamped_h as f32,
        );
    }
    Ok(Value::Null)
});

native_fn!(FnP5Circle, "p5_circle", 3, |_evaluator, args, cursor| {
    let x = args[0].check_num(cursor, Some("center x".into()))?;
    let y = args[1].check_num(cursor, Some("center y".into()))?;
    let diameter = args[2].check_num(cursor, Some("diameter".into()))?;
    if diameter <= 0.0 {
        return Ok(Value::Null);
    }
    let runtime = get_runtime(cursor)?;
    {
        let state = runtime.state();
        let mut lock = state.lock().unwrap();
        lock.draw_circle(x as f32, y as f32, diameter as f32);
    }
    Ok(Value::Null)
});

native_fn!(FnP5Ellipse, "p5_ellipse", 4, |_evaluator, args, cursor| {
    let x = args[0].check_num(cursor, Some("center x".into()))?;
    let y = args[1].check_num(cursor, Some("center y".into()))?;
    let width = args[2].check_num(cursor, Some("width".into()))?;
    let height = args[3].check_num(cursor, Some("height".into()))?;
    if width <= 0.0 || height <= 0.0 {
        return Ok(Value::Null);
    }
    let runtime = get_runtime(cursor)?;
    {
        let state = runtime.state();
        let mut lock = state.lock().unwrap();
        lock.draw_oval(
            x as f32,
            y as f32,
            (width / 2.0) as f32,
            (height / 2.0) as f32,
        );
    }
    Ok(Value::Null)
});

native_fn!(FnP5Line, "p5_line", 4, |_evaluator, args, cursor| {
    let x1 = args[0].check_num(cursor, Some("x1".into()))?;
    let y1 = args[1].check_num(cursor, Some("y1".into()))?;
    let x2 = args[2].check_num(cursor, Some("x2".into()))?;
    let y2 = args[3].check_num(cursor, Some("y2".into()))?;
    let runtime = get_runtime(cursor)?;
    {
        let state = runtime.state();
        let mut lock = state.lock().unwrap();
        lock.draw_line(x1 as f32, y1 as f32, x2 as f32, y2 as f32);
    }
    Ok(Value::Null)
});

native_fn!(FnP5Background, "p5_background", 3, |_evaluator, args, cursor| {
    let r = args[0].check_num(cursor, Some("red".into()))?;
    let g = args[1].check_num(cursor, Some("green".into()))?;
    let b = args[2].check_num(cursor, Some("blue".into()))?;
    let color = color_from_rgb(r, g, b);
    let runtime = get_runtime(cursor)?;
    {
        let state = runtime.state();
        let mut lock = state.lock().unwrap();
        lock.background(color);
    }
    Ok(Value::Null)
});

native_fn!(FnP5Fill, "p5_fill", 3, |_evaluator, args, cursor| {
    let r = args[0].check_num(cursor, Some("red".into()))?;
    let g = args[1].check_num(cursor, Some("green".into()))?;
    let b = args[2].check_num(cursor, Some("blue".into()))?;
    let color = color_from_rgb(r, g, b);
    let runtime = get_runtime(cursor)?;
    {
        let state = runtime.state();
        state.lock().unwrap().fill_color = Some(color);
    }
    Ok(Value::Null)
});

native_fn!(FnP5Stroke, "p5_stroke", 3, |_evaluator, args, cursor| {
    let r = args[0].check_num(cursor, Some("red".into()))?;
    let g = args[1].check_num(cursor, Some("green".into()))?;
    let b = args[2].check_num(cursor, Some("blue".into()))?;
    let color = color_from_rgb(r, g, b);
    let runtime = get_runtime(cursor)?;
    {
        let state = runtime.state();
        state.lock().unwrap().stroke_color = Some(color);
    }
    Ok(Value::Null)
});

native_fn!(FnP5NoFill, "p5_no_fill", 0, |_evaluator, _args, cursor| {
    let runtime = get_runtime(cursor)?;
    {
        let state = runtime.state();
        state.lock().unwrap().fill_color = None;
    }
    Ok(Value::Null)
});

native_fn!(FnP5NoStroke, "p5_no_stroke", 0, |_evaluator, _args, cursor| {
    let runtime = get_runtime(cursor)?;
    {
        let state = runtime.state();
        state.lock().unwrap().stroke_color = None;
    }
    Ok(Value::Null)
});

native_fn!(FnP5StrokeWeight, "p5_stroke_weight", 1, |_evaluator, args, cursor| {
    let weight = args[0].check_num(cursor, Some("weight".into()))?;
    if weight <= 0.0 {
        return Err(RuntimeEvent::error(
            "stroke weight must be positive".into(),
            cursor,
        ));
    }
    let runtime = get_runtime(cursor)?;
    {
        let state = runtime.state();
        state.lock().unwrap().stroke_weight = weight.max(0.1) as f32;
    }
    Ok(Value::Null)
});

native_fn!(FnP5Size, "p5_size", 2, |_evaluator, args, cursor| {
    let width = convert_len(
        args[0].check_num(cursor, Some("width".into()))?,
        "width",
        cursor,
    )?;
    let height = convert_len(
        args[1].check_num(cursor, Some("height".into()))?,
        "height",
        cursor,
    )?;

    let runtime = get_runtime(cursor)?;
    {
        let state = runtime.state();
        let mut lock = state.lock().unwrap();
        lock.width = width;
        lock.height = height;
        lock.buffer = vec![0; width * height * 4];
        lock.dirty = true;
    }
    runtime.send(P5Command::Resize(width as u32, height as u32));
    Ok(Value::Null)
});

native_fn!(FnP5Setup, "p5_setup", 1, |_evaluator, args, cursor| {
    let callback = ensure_callable(&args[0], cursor, "setup callback")?;
    P5_CALLBACKS.with(|cbs| {
        cbs.borrow_mut().setup = Some(callback);
    });
    Ok(Value::Null)
});

native_fn!(FnP5Draw, "p5_draw", 1, |_evaluator, args, cursor| {
    let callback = ensure_callable(&args[0], cursor, "draw callback")?;
    P5_CALLBACKS.with(|cbs| {
        cbs.borrow_mut().draw = Some(callback);
    });
    Ok(Value::Null)
});

native_fn!(FnP5Run, "p5_run", 0, |evaluator, _args, cursor| {
    let runtime = ensure_runtime(cursor)?;
    let state = runtime.state();

    let mut callbacks = P5_CALLBACKS.with(|cbs| cbs.borrow().clone());
    if callbacks.setup.is_none() {
        callbacks.setup = lookup_env_callable(evaluator, "setup", cursor)?;
    }
    if callbacks.draw.is_none() {
        callbacks.draw = lookup_env_callable(evaluator, "draw", cursor)?;
    }
    P5_CALLBACKS.with(|cbs| *cbs.borrow_mut() = callbacks.clone());

    if let Some(cb) = callbacks.setup.clone() {
        let _guard = runtime.begin_frame();
        cb.call(evaluator, vec![], cursor)?;
    }

    loop {
        let open = { state.lock().unwrap().open };
        if !open {
            break;
        }

        if let Some(cb) = callbacks.draw.clone() {
            let _guard = runtime.begin_frame();
            cb.call(evaluator, vec![], cursor)?;
        }

        thread::sleep(Duration::from_millis(16));
    }

    Ok(Value::Null)
});
