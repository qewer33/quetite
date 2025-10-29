# queitite

a simple interpreted scripting language

```rb
fn sq(n) = n*n

fn main() do
    var a = sq(10)
    var b = true
    b = false

    # this is a comment

    if a == 100 do
        print("poggers")
    end

    print(a)
end

main()
```

# Language Reference

## Syntax

Queitite has a very simple and straightforward syntax.

### Expressions

Expressions in queitite are similar to any C-style language. Queitite supports all of the standard arithmetic and boolean operators along with less common ones such as "**" (power) and "??" (nullish coalescing). It also supports logical OR and logical AND (as keywords, "or" and "and").

```rb
# basic arithmetic
5 + 5 * (2 - 3) / 10

# power operator
2**3

# boolean ops
5 <= 10 or 10 == 10 and 3 > 5

# nullish coalescing
null ?? "not null"

var a
a ?? 10
```

### Declarations and Assignment

Queitite is a fully dynamically typed language. It supports variable and function declarations with the "var" and "fn" keywords. There are 5 value types in queitite along with a Null value:

- Num: 64 bit floating point number format
- Str: Dynamically allocated string format
- Bool: Boolean type that can either be true or false
- List: Dynamically allocated list/array that can hold any number of elements of any type
- Callable: Functions are first class values as callables

Variables already declared can be re-assigned to any type.

```rb
# variable declarations
var a

var b = true
b = "amogus"

var n = 10
var pi = 3.14

# function declarations
fn square(n) do
    return n*n
end

print(square(10))
```

# Implementation

The queitite interpreter is a 3 stage interpreter with a Lexer that tokenizes the source code, a Parser that parses the tokens into an AST (Abstract Syntax Tree) and an Evaluator that walks the AST and executes it.

## Lexer

The lexer is implemented as a simple tokenizer.

Run the interpreter with the `--dump-tokens` flag to see the token output of a source file without executing it.

For more information, see the README.md file inside src/lexer.

## Parser

The parser is implemented as a recursive descent parser.

Run the interpreter with the `--dump-ast` flag to see the AST output of a source file without executing it.

For more information, see the README.md file inside src/parser.

## Evaluator

The evaluator is implemented as a tree-walk interpreter.

For more information, see the README.md file inside src/evaluator.