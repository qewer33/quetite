# Quetite Standard Library API Reference

This document aims to be a complete API reference for the Quetite standard library.

## Overview

The Quetite standard library consists of global functions, type prototypes and object namespaces.

## Globals

- `print(value)`  
  Writes `value` to stdout without appending a newline. Returns `Null`. Accepts any type; formatting uses the value’s `to_string` representation.

- `println(value)`  
  Writes `value` to stdout followed by a newline. Returns `Null`. Accepts any type.

- `read() -> Str`  
  Blocks on stdin, reads a line, trims the trailing newline, and returns it as `Str`. Throws `IOErr` if stdin read fails.

- `err(kind: Str, msg: Str) -> throws`  
  Immediately raises a runtime error of the given `kind` (`TypeErr`, `NameErr`, `ArityErr`, `ValueErr`, `NativeErr`, `IOErr`, `UserErr`) with message `msg`. Use inside `throw` or directly to abort execution.

## Type Prototypes

### Value

The methods on the Value prototype are available on every value regardless of it's type.

- `type() -> Str`  
  Returns the type name (`Null`, `Bool`, `Num`, `Str`, `List`, `Dict`, `Fn`, `Obj`, or object name for instances).

- `type_of(type: Str) -> Bool`  
  Case-insensitive comparison of the receiver’s type with `type`. Returns `true` on match, `false` otherwise.

- `type_check(type: Str) -> Bool or throws`  
  Returns `true` if the receiver’s type matches `type`; otherwise throws `TypeErr`. Useful for validating inputs.

### Num

- `abs() -> Num`  
  Absolute value of the number.

- `round() -> Num`  
  Rounds to the nearest integer (ties to nearest even per Rust `f64::round` semantics).

- `ceil() -> Num`  
  Rounds up to the smallest integer greater than or equal to the value.

- `floor() -> Num`  
  Rounds down to the largest integer less than or equal to the value.

- `clamp(min: Num, max: Num) -> Num`  
  Returns the receiver clamped to the inclusive range `[min, max]`. If `min > max`, returns `Null`.

- `to_str() -> Str`  
  Converts the number to its string representation.

### Bool

- `to_num() -> Num`  
  Converts `true` to `1` and `false` to `0`.

### Str

- `parse_num() -> Num | Null`  
  Attempts to parse the string as `f64`. On success returns the numeric value; on failure returns `Null` (no error thrown).

- `len() -> Num`  
  Returns the character length of the string.

- `repeat(n: Num) -> Str`  
  Repeats the string `n` times and returns the new string. `n` is truncated to `usize`; throws if `n` is not a number.

- ANSI color/style helpers (return new styled strings):
  Foreground: `black()`, `red()`, `green()`, `yellow()`, `blue()`, `magenta()`, `cyan()`, `white()`, `bright_black()`, `bright_red()`, `bright_green()`, `bright_yellow()`, `bright_blue()`, `bright_magenta()`, `bright_cyan()`, `bright_white()`.  
  Styles: `bold()`, `dim()`, `italic()`, `underline()`, `blink()`, `reverse()`, `strikethrough()`.  
  Background: `on_black()`, `on_red()`, `on_green()`, `on_yellow()`, `on_blue()`, `on_magenta()`, `on_cyan()`, `on_white()`.

### List

- `len() -> Num`  
  Number of elements.

- `push(value)`  
  Appends `value` to the end. Mutates the list in place.

- `pop() -> Value | Null`  
  Removes and returns the last element. Returns `Null` if the list is empty.

- `insert(index: Num, value)`  
  Inserts `value` at `index` (0‑based). If `index` is beyond the end, it will panic (runtime error).

- `remove(index: Num)`  
  Removes the element at `index`. Panics if out of bounds.

- `last() -> Value | Null`  
  Returns the last element, or `Null` if empty.

- `first() -> Value | Null`  
  Returns the first element, or `Null` if empty.

- `contains(value) -> Bool`  
  Returns `true` if any element equals `value` (uses value equality).

### Dict

Keys must be hashable (`Null`, `Bool`, `Num`, `Str`). Values can be any type.

- `len() -> Num`  
  Number of key/value pairs.

- `contains(key) -> Bool`  
  `true` if `key` exists, `false` otherwise. Throws `TypeErr` if key is not hashable.

- `insert(key, value)`  
  Sets `value` for `key`, inserting or replacing.

- `remove(key) -> Value | Null`  
  Deletes `key` and returns the removed value, or `Null` if missing.

- `get(key) -> Value | Null`  
  Returns the value for `key`, or `Null` if missing.

- `keys() -> List`  
  Returns a list of keys as values (`Null`, `Bool`, `Num`, `Str`).

- `values() -> List`  
  Returns a list of values.

## Namespaces

### Sys

System utilities for time and environment. All functions are static: `Sys.name()`.

- `Sys.clock() -> Num`  
  Milliseconds since UNIX epoch (`f64`).

- `Sys.sleep(ms: Num)`  
  Sleeps for `ms` milliseconds. Non‑numeric arguments are ignored (returns `Null`).

- `Sys.env(name: Str) -> Str | Null`  
  Returns the value of environment variable `name`, or `Null` if unset or inaccessible.

- `Sys.args() -> List<Str>`  
  Process arguments, including the program name at index 0.

- `Sys.cwd() -> Str`  
  Current working directory. Throws `IOErr` on failure.

### Math

Math helpers; all arguments and return values are `Num` (`f64`). Functions throw on invalid domains (e.g., log of non‑positive values).

- `Math.sin(x)`, `cos(x)`, `tan(x)`
  Trigonometric functions

- `Math.asin(x)`, `acos(x)`, `atan(x)`, `atan2(y, x)`
  Inverse trigonometric functions

- `Math.sqrt(x)`, `Math.cbrt(x)`, `Math.root(x, n)`
  Root functions

- `Math.exp(x)`

- `Math.ln(x)` — `x` must be > 0.
- `Math.log10(x)` — `x` must be > 0.
- `Math.log(value, base)` — `value` > 0; `base` > 0 and != 1.

- `Math.pow(base, exp)`

- `Math.hypot(a, b)`

- `Math.pi()`, `Math.tau()`, `Math.e()`
  Constant functions, return the respective constant as `Num`.

### Rand

Random number/string/list utilities (uniform RNG).

- `Rand.num() -> Num`  
  Random float in `[0, 1)`.

- `Rand.bool() -> Bool`  
  Random boolean.

- `Rand.list(list: List) -> Value`  
  Returns a random element from `list`. Errors on empty list or non‑list input.

- `Rand.string(len: Num) -> Str`  
  Random alphanumeric string of length `len`. `len` must be a non‑negative integer; throws otherwise.

- `Rand.range(min: Num, max: Num) -> Num`  
  Random float in `(min, max)`. Throws if `max <= min`.

- `Rand.int(min: Num, max: Num) -> Num`  
  Random integer in `[min, max]`. Bounds must be integers; throws if `max < min`.

### Term

Terminal control and non‑blocking input (crossterm). Many functions may throw `IOErr` on terminal failures. All calls are static: `Term.name()`.

- `Term.size() -> [Num width, Num height]`  
  Returns terminal columns and rows.

- `Term.get_input() -> KeyInput | Null`  
  Non‑blocking poll for a key event. Returns `Null` if no input. On key press, returns a `KeyInput` object with:
  - `key()` -> `Str` — key name (e.g., "Char('a')", "Enter", "Tab").
  - `ctrl()` -> `Bool` — `true` if Ctrl held.
  - `shift()` -> `Bool` — `true` if Shift held (also true for BackTab).
  - `alt()` -> `Bool` — `true` if Alt held.

- `Term.cursor_hide()` / `Term.cursor_show()` -> `Null`  
  Hide or show the cursor.

- `Term.cursor_move(x: Num, y: Num) -> Null`  
  Move cursor to `(x, y)` (0‑based). Throws `IOErr` on terminal failure.

- `Term.raw_enable()` / `Term.raw_disable()` -> `Null`  
  Enable/disable raw mode (keyboard events are delivered immediately).

- `Term.clear()` -> `Null`  
  Clear the entire screen and move cursor to (0,0).

- `Term.clear_line()` -> `Null`  
  Clear the current line.

- `Term.put(x: Num, y: Num, str)` -> `Null`  
  Move cursor to `(x, y)`, print `str` (converted to string if needed), cursor stays where it ends.

- `Term.write(str)` -> `Null`  
  Print `str` at current cursor position and advance cursor.

- `Term.set_title(str)` -> `Null`  
  Set the terminal/window title.

- `Term.flush()` -> `Null`  
  Flush stdout buffer manually.

### Tui

Terminal UI toolkit (ratatui-based) providing layout, widgets, and event handling for building interactive TUIs. See `src/evaluator/natives/tui.rs` for the full widget and API surface. Not complete yet.

### P5

Creative coding / simple graphics API inspired by Processing/p5.js. Provides drawing primitives, animation loop, and input handling. See `src/evaluator/natives/p5.rs` for the full API. Not complete yet.
