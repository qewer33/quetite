![banner](https://github.com/qewer33/queitite/blob/main/assets/readme_banner.png?raw=true)

Quetite is a small interpreted scripting language with a friendly, Ruby-like syntax, dynamic typing, and an stdlib of native helpers. It is implemented as a classic three-stage interpreter (lexer -> parser -> evaluator) in Rust.

```rb
println("Hello Quetite!")
```

> [!WARNING]  
> Quetite is still a WIP (work in progress) language. Expect breaking changes!

## Features
- [x] Easy and familiar scripting language syntax, no semicolons, no indentation rules
- [x] Fully dynamic type system (with types like `Bool`, `Num`, `Str` etc...)
- [x] First class functions (`Callable` type) and objects with constructors, static and bound methods (`Obj` type)
- [x] Fully dynamic lists and dictionaries (`List` and `Dict` types)
- [x] Internal prototype methods on primitives (`Str.len()`, `List.push()`, `Dict.keys()`, etc.)
- [x] Familiar control flow: `if/else`, `while`, `for` over iterables, `match`
- [x] Friendly and easy to understand errors!
- [x] Runtime error handling system via `throw` and `try/catch/ensure`
- [x] Misc scripting features such as: ranges (`..` and `..=` operators), list/string slicing (`str[a..b]`), nullish coalescing (`a ?? b`), power (`a**b`) and ternary (`cond ? a : b`) operators
- [x] Truthiness rules (`false`, `Null`, `0` are falsy; everything else is truthy)
- [x] Powerful and extensive stdlib (`Sys`, `Math`, `Rand`, `Term` etc.)
- [x] Ability to include other scripts inside a script via `use`
- [x] Fully interactive REPL with interactive `help` command
- [ ] Ratatui bindings for the stdlib to create fun TUI apps! (half implemented, full implementation coming soon!)
- [ ] Processing/p5.js like simple creative coding and graphics API for the stdlib to create fun games and creative programs! (half implemented, full implementation coming soon!)

## Quick Start

Prereqs: Rust toolchain installed via `rustup`.

Build and run a script:
```sh
cargo run path/to/script.qte
```

Run the example `snake.qte`:
```sh
cargo run examples/snake.qte
```

Run the interactive REPL:
```sh
cargo run
```

Use the `help` command inside the REPL to interactively explore "The Quetite Language Reference" and "API Reference" documentations. Also check out the examples folder to see other examples!

## Example

```rb
# hello.qte

var who = "world"
var nums = 0..=4
var facts = {
    "name": who,
    "lucky": nums[3],
}

fn greet(name) do
    println("hello, " + name + "!")
end

greet(facts["name"])

for n in nums do
    if n % 2 == 0 do
        println(n.to_str() + " is even")
    else do
        println(n.to_str() + " is odd")
    end
end
```

## Quetite at a Glance

- **Values & Prototypes**  
  `type()`, `type_of()`, `type_check()` on any value; conversions via `to_*()` helpers.

- **Strings**  
  Indexing and slicing; `len()`, `repeat(n)`, and terminal color/style helpers.

- **Lists**  
  Dynamic arrays with `len()`, `push()`, `pop()`, `insert(i, v)`, `remove(i)`, `first()`, `last()`, `contains(v)`.

- **Dicts**  
  Hash maps keyed by `Null/Bool/Num/Str`; `len()`, `contains(k)`, `insert(k, v)`, `remove(k)`, `get(k)`, `keys()`, `values()`.

- **Control Flow**  
  `if/else`, `while`, `for value, index in iterable`, `match`, ternary `cond ? a : b`, ranges `a..b` and `a..=b` with optional `step`, slicing with ranges.

- **Errors**  
  `try/catch/ensure` and `throw`; internal error types include `TypeErr`, `NameErr`, `ArityErr`, `ValueErr`, `NativeErr`, `IOErr`, `UserErr`.

- **Objects & Functions**  
  First-class functions; objects with optional `init()` constructor, static methods, and bound methods using `self`.

## Repository Layout
- `examples` - example Quetite scripts
- `src/lexer` - tokenizer
- `src/parser` - recursive descent parser producing AST
- `src/evaluator` - tree-walk interpreter and stdlib natives
- `REFERENCE.md` - full language reference
