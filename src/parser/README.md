# Parser

### Grammar

```js
program        → statement* EOF ;


declaration    → varDecl
               | statement ;

varDecl        → IDENTIFIER ( "=" expression )? EOL ;

statement      → exprStmt
               | printStmt
               | block ;

exprStmt       → expression EOL ;
printStmt      → "print" expression EOL ;
block          → "do" declaration "end" ;

expression     → assignment ;
assignment     → IDENTIFIER "=" assignment
               | equality ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" | "**" ) unary )* ;
unary          → ( "!" | "-" ) unary
               | primary ;
primary        → NUMBER | STRING | "true" | "false" | "nil"
               | "(" expression ")"
               | IDENTIFIER ;
```