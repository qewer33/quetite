# Parser

### Grammar

```js
program        → statement* EOF ;

declaration    → classDecl
               | funDecl
               | varDecl
               | statement ;

classDecl      → "obj" IDENTIFIER "{" function* "}" ;
funDeclr       → "fn" function ;
function       → IDENTIFIER "(" parameters? ")" block ;
parameters     → IDENTIFIER ( "," IDENTIFIER )* ;
varDeclr       → "var" IDENTIFIER ( "=" expression )? EOL ;
varDeclrHeader → "var" IDENTIFIER "=" expression ;

statement      → exprStmt
               | ifStmt
               | forStmt
               | returnStmt
               | breakStmt
               | continueStmt
               | whileStmt
               | block ;

exprStmt       → expression EOL ;
ifStmt         → "if" expression statement
               ( "else" statement )? ;
forStmt        → "for" ( varDecl | exprStmt | "and" )
                 expression? "and"
                 expression? statement ;
returnStmt     → "return" expression EOL ;
breakStmt      → "break" EOL ; 
continueStmt   → "continue" EOL ; 
whileStmt      → varDeclrHeader? "while" expression ("step" assignment)? statement ;
block          → "do" declaration "end" ;

expression     → assignment ;
assignment     → ( call "." )? IDENTIFIER ("=" | "+=" | "-=" ) assignment
               | logic_or ;
logic_or       → logic_and ( "or" logic_and )* ;
logic_and      → equality ( "and" equality )* ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" | "**" | "??" ) unary )* ;
unary          → ( "!" | "-" ) unary | call ;
arguments      → expression ( "," expression )* ;
call           → primary ( "(" arguments? ")" | "." IDENTIFIER )* ;
list           - "[" arguments? "]" ;
primary        → NUMBER | STRING | "true" | "false" | "nil"
               | "(" expression ")"
               | IDENTIFIER ;
```