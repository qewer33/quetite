# The Quetite Language Reference

This document aims to be a quick and simple reference guide for the Quetite language. It provides the necessary details for the programmer to get started with Quetite without going into too much unnecessary detail. This document is mainly about the language syntax and features, for the full language stdlib API documentation, see the *API reference*.

## Quick Start

Start by creating a new file with the `.qte` extension (eg. `hello.qte`). You can choose the Ruby language as the syntax highlighting in your editor (doesn't highlight everything correctly but it's mostly decent since Quetite syntax is similar to Ruby). Write the following line inside the file:

```rb
println("Hello World")
```

To execute the file, pass it as an argument to the Quetite interpreter:

```sh
quetite hello.qte
```

You just wrote and ran your first Quetite script, congrats! :D

You can also explore Quetite with the interactive Quetite shell (REPL). Start an interactive shell session by invoking the Quetite interpreter with no arguments:

```sh
quetite
```

## Lexical Structure

### Comments

Comments in Quetite start with the hash (`#`) character and continue to the end of the line. Quetite does not currently support multi-line comment blocks; use multiple single-line comments instead.

```rb
# hello Quetite!
# this is a comment
```

### Identifiers

Identifiers are names used for variables, functions and objects. Identifiers in Quetite can only contain letters, numbers or underscores (`_`). Identifiers cannot start with a number. Identifiers are case sensitive.

For variable and function identifiers, `snake_case` is recommended. For object identifiers, `PascalCase` is preffered.

### Whitespaces & Newlines

Quetite is a line oriented language, statements are terminated by newlines. Indentation and whitespaces are ignored and have no effect but proper indentation of Quetite code is recommended for readability.

## Type System

Quetite is a dynamically typed language, meaning the types aren't explicitly known at compile time but are rather evaluated at runtime.

Quetite has 8 value types:
- **Null**: The `Null` literal, representing the absence of a value.
- **Bool**: The boolean value type, can either be `true` or `false`.
- **Str**: The string type, holds a dynamically allocated string value.
- **List**: The list type, can hold any amount of any type of elements.
- **Dict**: The dictionary type, holds key-value pairs of elements.
- **Callable**: The callable type, holds a function or method definition.
- **Obj**: The obj type, holds an object definition.
- **ObjInstance**: Each object instance acts like it's own type but internally, they're represented as a single object instance type.

### Truthiness

In Bool contexts (such as the conditions in an `if` or `while` statement), non-Bool values are converted to a Bool value via the internal truthiness table. `Null` and `0` are considered to be falsy while everything else (including all Str and non-zero Num values) is truthy.

### Type Prototypes

All values share an internal `Value` prototype which holds methods that can be called from all values regardless of it's type (methods such as `type()`). The `Bool`, `Num`, `Str`, `List` and `Dict` types also have their own respective internal prototypes. Check out the *API reference* to see which functions are defined for which prototype.

### Runtime Type Checking

The following methods defined in the Value prototype can be used for runtime type checking in Quetite:

- `type()`: Returns the type of the value as an Str.
- `type_of(type)`: Expects a type as an Str. Returns `true` if the type of the value matches the given type, `false` otherwise.
- `type_check(type)`: Expects a type as an Str. Returns `true` if the type of the value matches the given type, throws a `TypeErr` otherwise. This function is recommended for ensuring types of function parameters.

### Type Conversions

Type conversions in Quetite are done with the `to_*()` methods provided in value prototypes. The Num prototype for example, provides the `to_str()` and `to_bool()` methods to convert the Num value to an Str and a Bool respectively. 

### Types in Detail

#### Null

The null type that can only be the Null literal. The Null literal represents an absent or unknown value. It's also the value returned from functions without a `return` statement (or an empty one).

#### Bool

The boolean type that can either be `true` or `false`. It's the value type returned by boolean operations.

```rb
var a = true

# b = false
var b = !a

# c = true
var c = a or b
```

#### Num

The number type holds integer and floating point numbers. Internally it's a 64 bit float. The Num prototype provides many functions to make it easier to work with Nums.

```rb
# an integer and a float
var int = 10
var float = 10.25

# rounding a float to an integer
# a = 10
var a = 10.36.round()
```

#### Str

The string type that holds a dynamically allocated string. String literals are created with the double quote character (`""`). Str values can be indexed with the indexing (`value[i]`) syntax, the index should either be a Num or a List of Nums. The Str prototype provides many functions to make it easier to work with Strs.

```rb
# defining an Str
var str = "hello Quetite!"

# indexing an Str
# prints "h"
println(str[0])

# length of an Str
# prints 13
println(str.len())
```

#### List

The list type that holds a dynamically allocated list. A List can hold any type and any number of elements, it can also hold mixed types of elements. List literals are created with square braces (`[]`) and the list elements are separated with commas (`,`). Str values can be indexed with the indexing (`value[i]`) syntax, the index should either be a Num or a List of Nums. The List prototype provides many functions to make it easier to work with Lists.

```rb
# defining a List
var fruits = ["Apple", "Orange", "Banana"]
var stuff = ["Among Us", 12, true, Null]

# indexing a List
# prints "orange"
println(fruits[1])

# length of a List
# prints 4
println(stuff.len())
```

#### Dict

The dict type holds a dynamically allocated dictionary/map of elements in key-value pairs. Internally, it's represented as a HashMap; thus it can only have "hashable" value types as keys (`Null`, `Bool`, `Num` and `Str`). It can hold any type as a value. Dict literals are created with key-value pairs (`key: value`) defined inside curly braces (`{}`) and seperated by commas (`,`). Dict values can be indexed with the indexing (`value[i]`) syntax, the index should be one of the aforementioned hashable value types. The Dict prototype provides many functions to make it easier to work with Dicts.

```rb
# defining a Dict
var stuff = {
    "amogus": "sus",
    Null: true,
    5: "five",
    false: 0
}

# indexing a Dict
println(stuff["amogus"])
println(stuff[Null])
println(stuff[5])

# length of a Dict
# prints 4
println(stuff.len())
```

#### Callable

Functions in Quetite are first-class as the Callable type, meaning they can be assigned to variables and passed around as arguments to other functions or as object fields.

```rb
# an example of passing a function to another function as callback
fn read_and_do(callback) do
    var input = read()
    callback(input)
end

# reads input from the user and prints it
read_and_do(println)
```

#### Obj

Objects are also first-class in Quetite, just like functions. Object definitions can be assigned to variables, passed around as function parameters and can be returned from functions.

## Grammar

Quetite has two distinct grammar structures, statements and expressions. Expressions get evaluated to a value while statements get executed without any output value.

### Expressions

#### Arithmetic

Arithmetic expressions in Quetite are very similar with other mainstream scripting languages. All of the classic arithmetic operators are included, along with some less common ones like the power (`a**b`) operator.

| **Expression** | **Operator** | **Usage** |
|----------------|--------------|-----------|
| Addition       | +            | a + b     |
| Subtraction    | -            | a - b     |
| Multiplication | \*           | a \* b    |
| Division       | /            | a / b     |
| Modulo         | %            | a % b     |
| Power          | \*\*         | a\*\*b    |

The Num type supports every kind of arithmetic operation while Str supports only addition (string concatenation). Other types don't support any arithmetic operations.

#### Boolean

Boolean expressions in Quetite are very similar with other mainstream scripting languages. All of the classic boolean operators are included, along with some less common ones like the nullish coalescing (`a ?? b`) operator.

| **Expression**     | **Operator** | **Usage** |
|--------------------|--------------|-----------|
| Equal              | ==           | a == b    |
| Not Equal          | !=           | a != b    |
| Greater Equal      | >=           | a >= b    |
| Lesser Equal       | <=           | a <= b    |
| Greater            | >            | a > b     |
| Lesser             | <            | a < b     |
| Negation           | !            | !a        |
| Logical And        | and          | a and b   |
| Logical Or         | or           | a or b    |
| Nullish Coalescing | ??           | a ?? b    |

The nullish coalescing (`a ?? b`) operator is a special operator that returns `b` if `a == Null`, returns `a` otherwise. It supports all types, `a` and `b` can also be different types.

The equal operation is supported by all value types but only works if `a` and `b` are the same type. The logical and/or operators are supported on every type via the truthiness table. Comparison operators are only supported on Num values. All the boolean operations (excluding nullish coalescing) evaluate to a Bool value.

#### Assignment

An assignment epression is used to re-assign the value of an already defined (see Variable Declaration in Statements). Quetite has 5 different assignment operations:

| **Operation**     | **Operator** | **Usage** |
|-------------------|--------------|-----------|
| Normal Assignment | =            | a = b     |
| Add Assign        | +=           | a += b    |
| Sub Assign        | -=           | a -= b    |
| Increment         | ++           | a++       |
| Decrement         | --           | a--       |

The normal assignment operation is supported by all types and the two values do not have to be of the same type. The add assign operation is supported by Num, Str and List types. The other operations are only supported by the Num type.

#### Group

A group is used to change the evaluation order of expressions, it's defined with a set of parentheses (`()`).

```rb
var a = 5 * (4 + 3)

# call Num.round() on the resulting expression
var b = (5.32 * a).round()
```

#### Ternary

The classic ternary (`condition ? true : false`) expression is a simple way to conditionally return a value. It works exactly the same way it does in C.

```rb
var a = 5
var b = 10

# assign c to the smaller number
var c = a < b ? a : b
```

#### Range

A range expressions is syntax sugar for creating List's of ordered numbers. Ranges are created with the `..` and `..=` operators, the `..=` operator includes the end value in the range meanwhile the `..` operator doesn't. A range can also have an optional `step` expression that specifies the "step" (increment amount) between the range values.

```rb
# a, b and c are the same!
var a = [0, 1, 2]
var b = 0..3
var c = 0..=2

# range with a step
var a = [0, 2, 4, 8]
var b = 0..=8 step 2
```

The range operators can also be used to "slice" Lists and Strs.

```rb
# slicing an Str
var a = "amogus"
a[0..3] = "sus"
# prints "susgus"
println(a)
# prints "gus"
println(a[3..a.len()])

# slicing a List
var b = [0, 1, 2, 3]
# prints [0, 1]
println(b[0..2])
```

### Statements

#### Block

A block opens a new lexical scope. Starts with the `do` keyword and ends with the `end` keyword. Blocks are usually used as bodies for other statements.

```rb
# create a global variable
var a = 10

# open a new scope
do
    # create a scope local variable
    var a = 20
    # prints 20
    println(a)
end

# prints 10
println(a)
```

#### If

The classic `if` statement used for conditional branching. Can be followed by `else` and/or `else if` when needed. 

```rb
if true do
    println("this will always run!")
end

# three-way comparison
var a = 10
if a < 5 do
    println("a is smaller than 5")
else if a > 5 do
    println("a is greater than 5")
else do
    println("a is equal to 5")
end
```

#### Match

The match statement (also called `switch` or `case` in other languages) is used to check a value against a list of other values and execute a stetement if they match. It can be used to replace a series of `if...else` statements. The syntax is `match value do <arms> end`. A match arm is a value followed by a statement (can be a block or a single line statement, see below examples). A match statement can have an optional `else` branch at the end which runs if nothing matches the value.

```rb
# matching a variable against different values
var a = 0

match a do
    0 do
        println("a is zero!")
    end
    1 do
        println("a is one!")
    end
    "among us" do
        println("a is sus???")
    end
end
```

```rb
# simple calculator program using a compact match
print("Enter num a: ")
var a = read().parse_num()
print("Enter num a: ")
var b = read().parse_num()

print("Enter operation (+-*/): ")
var op = read()

print("Result: ")
match op do
    "+" println(a + b)
    "-" println(a - b)
    "*" println(a * b)
    "/" println(a / b)
else println("invalid operation")

```

#### While

The classic `while` loop used for conditional looping. While loops in Quetite also have special syntax for emulating C-style for loops in a single line with a variable declaration preceding the `while condition` part and a following `step` statement (see the example below).

The `break` and `continue` statements can be used inside a while loop to control loop iterations.

```rb
# infinite loop
while true do
    println("Quetite is cool!")
end

# emulating a C-style for loop
var i = 0
while i < 10 do
    println("Quetite is cool!")
    i++
end

# special syntax for emulating C-style for loops
var i = 0 while i < 10 step i++ do
    println("Quetite is cool!")
end
```

#### For

For loops in Quetite are used to iterate over iterable values (List and Str) with the `for value, index in list` syntax. The `index` identifier can be omitted if not required.

The `break` and `continue` statements can be used inside a for loop to control loop iterations.

```rb
# iterating over a list
for v, i in ["apple", "orange", "banana"] do
    println(i)
    println(v)
end

# iterating over a range (ranges evaluate to a List)
for i in 0..10 do
    println(i)
end
```

#### Try and Throw

The classic `try...catch...ensure` statement combo that is used for catching runtime errors. The catch statement can have optional identifiers for accessing the error type and value (eg. `catch e, v`). The `ensure` (also called `finally` in other languages) statement always runs, can be omitted if not needed.

The classic `throw` statement can be used for throwing runtime errors. The statement expects a value to be thrown (can be any type). For throwing internal error types (see below), the `err(type, message)` function can be used in combination with `throw` (see below examples).

Quetite has the following internal error types:
- **TypeErr**: The error thrown for type mismatches.
- **NameErr**:  The error thrown for name mismatches, usually when an identifier can't be found.
- **ArityErr**: The error thrown for function arity (parameter count) mismatches.
- **ValueErr**: The error thrown for value mismatches (eg. when a funciton expecst an integer Num but a float is provided).
- **NativeErr**: The error thrown when a fatal error (panic) occurs in native stdlib functions.
- **IOErr**: The error thrown when IO operations fail.
- **UserErr**: The error thrown by the `throw` statement.

```rb
# throwing and catching a UserErr
try do
    println("throwing an error...")
    throw "random error"
catch e, v do
    println("error catched")
    # prints "UserErr"
    println(e)
    # prints "random error"
    println(v)
ensure do
    println("ensure ran")
end

# throwing and catching a ValueErr
try do
    throw err("ValueErr", "value err")
catch e do
    # prints "ValueErr"
    println(e)
end
```

#### Variable Declaration

Variables can be declared with the `var` keyword.

```rb
var a = 10

var name = "qewer33"

var list = [1, name, 37.42, true]
```

#### Function Declaration

Functions can be declared with the `fn` keyword, followed by the function name and arguments inside parentheses. Functions can take any statement as a body but a block (`do..end`) is preferred most of the time.

```rb
# a single line square function
fn square(n) return n*n

# same square method defined with a body
fn square(n) do
    return n
end

# function call
var a = square(10)
```

#### Object Declaration

Objects can be declared with the `obj` keyword, followed by the object name and body. Methods can be defined inside object bodies without any keywords. Methods that take `self` as an argument are *bound methods* that can only be called from an instance meanwhile methods without the special `self` value as an argument act as *static methods* that can be directly called from the object namespace. A custom constructor for the object can be defined with the `init()` method. Only one constructor is permitted.

```rb
obj Pos do
    # custom constructor definition
    init(x, y) do
        self.x = x
        self.y = y
        self.x = x
        self.y = y
    end

    # static method
    add(pos1, pos2) do
        return Pos(pos1.x + pos2.x, pos1.y + pos1.y)
    end

    # bound method
    print(self) do
        println(self.x)
        println(self.x)
    end
end

# object instantiation
var pos1 = Pos(0, 10)
var pos2 = Pos(20, 30)

# bound method call
pos1.print()

# static method call
var pos2 = Pos.add(pos1, pos2)
```

#### Use

The `use` statement makes it possible to import scripts inside other scripts. It expects an Str after the keyword as the path of the script to be loaded. When loading a script, the interpreter first interprets the script to be loaded and then loads everything in the resulting global environment of the script (variables, functions and object declarations) to the global environment of the current script.

```rb
# other.qte
var a = 10
```

```rb
# main.qte
use "other.qte"

# prints 10
println(a)
```

## Standard Library

The Quetite standard library (stdlib) consists of functions and objects that are defined and implemented natively inside the qutite interpreter (in Rust). They are available to use in every Quetite script without needing a `use` statement.

The standard library has 4 global functions (all of which have been mentioned before in the document):

- `println(val)`: Used to print a value to the terminal (standard output) with a line terminator (`\n`) at the end.
- `print(val)`: Same as `println` but doesn't print line terminator (`\n`).
- `read()`: Reads a line from the user (standard input) and returns it as an Str.
- `err(type, msg)`: Used for throwing internal error types with a message.

The standard library also has 7 global objects that act as namespaces for different API functions:

- `Sys`: Provides system related functions (such as `Sys.sleep(ms)`, `Sys.clock()` and functions for reading CLI arguments). 
- `Math`: Provides math related functions (such as `Math.sin(x)` and `Math.cos(x)`).
- `Rand`: Provides functions for generating random numbers or making randomized choices.
- `Term`: Provides terminal related functions.
- `Fs`: Provides filysystem related functions.
- `Tui`: A full API for creating TUIs (terminal user interfaces). Uses the very popular Rust TUI crate `ratatui` in the background.
- `P5`: A full API for creative coding and basic computer graphics. Mimics the very popular Processing and p5.js frameworks.

For the full stdlib API documentation, see the *API reference*.

## Appendix

### Appendix A: Keywords

- do
- end
- if
- else
- for
- while
- return
- break
- continue
- use
- self
- var
- and
- or
- step
- in
- fn
- obj
- throw
- try
- catch
- ensure

### Appendix B: BNF Grammar

```ebnf
program        → statement* EOF ;

declaration    → classDecl
               | funDecl
               | varDecl
               | statement ;

classDecl      → "obj" IDENTIFIER "do" function* "end" ;
funDeclr       → "fn" function ;
function       → IDENTIFIER "(" parameters? ")" block ;
parameters     → IDENTIFIER ( "," IDENTIFIER )* ;
varDeclr       → "var" IDENTIFIER ( "=" expression )? EOL ;
varDeclrHeader → "var" IDENTIFIER "=" expression ;

statement      → exprStmt
               | ifStmt
               | returnStmt
               | breakStmt
               | continueStmt
               | forStmt
               | whileStmt
               | block ;

exprStmt       → expression EOL ;
ifStmt         → "if" expression statement
               ( "else" statement )? ;
matchStmt      → "match" expression "do" 
               ( expression statement )* 
               ( "end" | ( "else" statement )? ) ;
returnStmt     → "return" expression EOL ;
throwStmt      → "throw" expression EOL ;
breakStmt      → "break" EOL ; 
continueStmt   → "continue" EOL ; 
tryStmt        → "try" statement "catch" IDENTIFIER statement ;
forStmt        → "for" IDENTIFIER ( "," IDENTIFIER )? "in" expression "do" statement ;
whileStmt      → varDeclrHeader? "while" expression ("step" assignment)? statement ;
useStmt        → "use" expression EOL ;
block          → "do" declaration "end" ;

expression     → assignment ;
assignment     → ( call "." )? IDENTIFIER ( ( "=" | "+=" | "-=" ) assignment | ( "++" | "--" ) )
               | ternary_or ;
ternary        → logic_r ( "?" expression ":" ternary )? ;
logic_or       → logic_and ( "or" logic_and )* ;
logic_and      → equality ( "and" equality )* ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" | "**" | "??" ) unary )* ;
unary          → ( "!" | "-" ) unary | call ;
arguments      → expression ( "," expression )* ;
call           → primary ( "(" arguments? ")" | "." IDENTIFIER )* ;
range          → expr ( ".." | "..=" ) expr ( "step" expr )? ; 
list           - "[" arguments? "]" ;
dict           - "{" ( expression ":" expression ( "," expression ":" expression  )* )? "}" ;
primary        → NUMBER | STRING | "true" | "false" | "Null"
               | "(" expression ")"
               | IDENTIFIER ;
```