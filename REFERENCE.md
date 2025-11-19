# The Quetite Language Reference

This document aims to be a quick and simple reference guide for the quetite language. It aims to provide the necessary details for the programmer to get started with quetite without going into too much unnecessary detail. Thi documents is only about the language syntax and features, for the full language stdlib API documentation, see the *API reference*.

## Quick Start

Start by creating a new file with the `.qte` extension (eg. `hello.qte`). Write the following line inside the file:

```rb
println("Hello World")
```

To execute the file, pass it as an argument to the quetite interpreter:

```sh
quetite hello.qte
```

You just wrote and ran your first quetite script, congrats! :D

## Type System

Quetite is a dynamically typed language, meaning the types aren't explicityly known at compile time but rather evaluted at runtime.

Quetite has 8 value types:
- **Null**: The `Null` literal, representing the absence of a value.
- **Bool**: The booleathatn value type, can either be `true` or `false`.
- **Num**: The number type, can hold integer or floating point numbers. Internally it's a 64 bit float.
- **Str**: The string type, holds a dynamically allocated string value.
- **List**: The list type, can hold any amount of any type of elements.
- **Callable**: The callable type, holds a function or method definition.
- **Obj**: The obj type, holds an object definition.
- **ObjInstance**: Each object instance acts like it's own type but internally, they're represented as a single object instance type.

### Thruthiness

In Bool contexts (such as the conditions in an `if` or `while` statement), non-Bool values are converted to a Bool value via the internal truthiness table. `Null` and `0` are considered to be falsy while everything else (including all Str and non-zero Num values) is truthy.

### Type Prototypes

All values share an internal `Value` prototype which holds methods that can be called from all values regardless of it's type (methods such as `type()`). The `Bool`, `Num`, `Str` and `List` types also have their own respective internal prototypes. Check out the *API reference* to see which functions are defined for which prototype.

### Runtime Type Checking

The following methods defined in the Value prototype can be used for runtime type checking in quetitie:

- `type()`: Returns the type of the value as an Str.
- `type_of(type)`: Expects a type as an Str. Returns `true` if the type of the value matches the given type, `false` otherwise.
- `type_check(type)`: Expects a type as an Str. Returns `true` if the type of the value matches the given type, throws a `TypeErr` otherwise. This function is recommended for ensuring types of function parameters.

## Grammar

Quetite has two distinct grammar structures, statements and expressions. Expressions get evaluated to a value while statements get executed without any output value.

### Expressions

#### Arithmethic

Arithmethic expressions in quetite are very similar with other mainstream scripting languages. All the nullish coalescing (`a ?? b`) operatorof the classic arithmetic operators are included, along with some less common ones like the power (`a**b`) operator.

| **Expression** | **Operator** | **Usage** |
|----------------|--------------|-----------|
| Addition       | +            | a + b     |
| Subtraction    | -            | a - b     |
| Multiplication | *            | a * b     |
| Division       | /            | a / b     |
| Modulo         | %            | a % b     |
| Power          | **           | a**b      |

The Num type supports every kind of arithmetic operation while Str supports only addition (string concatenation). Oter types don't support any arithmetic operations.

#### Boolean

Boolean expressions in quetite are very similar with other mainstream scripting languages. All of the classic arithmetic operators are included, along with some less common ones like the nullish coalescing (`a ?? b`) operator.

| **Expression**     | **Operator** | **Usage** |
|--------------------|--------------|-----------|
| Equal              | ==           | a == b    |
| Not Equal          | !=           | a != b    |
| Greater Equal      | >=           | a >= b    |
| Lesser Equal       | <=           | a <= b    |
| Greater            | >            | a > b     |
| Lesser             | <            | a < b     |
| Logical And        | and          | a and b   |
| Logical Or         | or           | a or b    |
| Nullish Coalescing | ??           | a ?? b    |

The nullish coalescing (`a ?? b`) operator is a special operator that returns `b` if `a == Null`, returns `a` otherwise. It supports all types, `a` and `b` can also be different types.

The equal operation is supported by all value types but only works if `a` and `b` are the same type. The logical and/or operators are supported on every type via the truthiness table. Comparison operators are only supported on Num values.

#### Group

A group is used to change the evaluation order of expressions, it's defined with a set of parantheses (`()`).

### Statements

#### Block

A block, opens a new lexical scope. Starts with the `do` keyword and ends with the `end` keyword. Blocks are usually used as bodies for other statements.

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

#### While

The classic `while` loop used for conditional looping. While loops in quetite also have special syntax for emulating C-style for loops in a single line with a variable declaration preceding the `while condition` part and a following `step` statement (see the example below).

```rb
# infinite loop
while true do
    println("quetite is cool!")
end

# emulating a C-style for loop
var i = 0
while i < 10 do
    i++
    println("quetite is cool!")
end

# special syntax for emulating C-style for loops
var i = 0 while i < 10 step i++ do
    println("quetite is cool!")
end
```

#### For

For loops in quetite are used to iterate over iterable values (List and Str) with the `for value, index in list` syntax. The `index` identifier can be omitted if not required.

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

The classic `try...catch...ensure` statement combo that is used for catching runtime errors. The catc statement can have optional identifiers for accessing the error type and value (eg. `catch e, v`). The `ensure` (also called `finally` in other languages) statement can be omitted if  not needed.

The classic `throw` statement can be used throwing runtime errors. The statement expects a value to be thrown (can be any type). For throwing internal error types (see below), the `err(type, message)` function can be used in combination with `throw` (see below examples).

Quetite has the following internal error types:
- **TypeErr**: The error thrown for type mismatches.
- **NameErr**:  The error thrown for name mismatches, usually when an identifier can't be found.
- **ArityErr**: The error thrown for function arity mismatches.
- **ValueErr**: The error thrown for value mismatches.
- **NativeErr**: The error thrown when a native a fatal error (panic) occurs in native stdlib functions.
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

Functions can be declared with the `fn` keyword, followed by the function name and argumens inside parantheses. Functions can take any statement as a body but a block (`do..end`) is preferred most of the time.

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

Objects can be declared with the `obj` keyword, followed by the object name and body. Methods can be defined inside object bodies without any keywords. Methods that take `self` as an argument are *bound methods* that can only be called from an instance meanwhile methods without the special `self` value as an argument act as *static methods* that can be direclt called from the object namespace. A custom constructor for the object can be defined with the `init()` method. Only one constructır is permitted.

```rb
obj Pos do
    # custom constructor definition
    init(x, y) do
        self.x = x;
        self.y = y;
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
