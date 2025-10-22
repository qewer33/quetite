# Parser

### Grammar

```js
program        → statement* EOF ;


declaration    → varDecl
               | statement ;

varDecl        → IDENTIFIER ( "=" expression )? EOL ;

statement      → exprStmt
               | ifStmt
               | forStmt
               | printStmt
               | whileStmt
               | block ;

exprStmt       → expression EOL ;
ifStmt         → "if" expression statement
               ( "else" statement )? ;
forStmt        → "for" ( varDecl | exprStmt | "and" )
                 expression? "and"
                 expression? statement ;
printStmt      → "print" expression EOL ;
whileStmt      → "while" "(" expression ")" statement ;
block          → "do" declaration "end" ;

expression     → assignment ;
assignment     → IDENTIFIER "=" assignment
               | logic_or ;
logic_or       → logic_and ( "or" logic_and )* ;
logic_and      → equality ( "and" equality )* ;
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