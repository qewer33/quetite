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