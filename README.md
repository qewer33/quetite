# queitite

a simple interpreted scripting language

```rb
sq(n) = n*n

main() = do
    a = sq(10)
    b = true

    # this is a comment

    if a == 100 do
        print("poggers")
    end

    print(a)
end

main()
```

# Implementation

## Lexer

### Tokens

```rs
pub enum Token {
    // types
    Num(String),
    Bool(bool),
    Str(String),
    // assign
    Assign,
    AddAssign,
    SubAssign,
    Incr,
    Decr,
    // arithmetic
    Add,
    Sub,
    Mult,
    Div,
    Pow,
    // bool ops
    Not,
    Equals,
    NotEquals,
    Greater,
    GreaterEquals,
    Lesser,
    LesserEquals,
    // symbols
    LParen,
The parser is implemented as a straightforward recursive descent parser.

### Grammer

```js
expression     → equality ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Dot,
    // other
    Keyword(String),
    Identifier(String),
    EOL,
    EOF
}
```

## Parser
 ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" | "**" ) unary )* ;
unary          → ( "!" | "-" ) unary
               | primary ;
primary        → NUMBER | STRING | "true" | "false" | "nil"
               | "(" expression ")" ;
```