# micron-rust
This variant of micron is derived from the original by @Z3RYX.<br/>
This variant focuses more on strong typing and modularization by having slightly different behaviours of some specific instructions. Also, better errors.<br/>
Keep in mind that this implementation is still in development so some features are subjected to change.

## Table of contents
- [micron-rust](#micron-rust)
  * [Usage](#usage)
  * [Specification](#specification)
    + [Data Types](#data-types)
    + [Slots](#slots)
    + [Instructions](#instructions)
    + [Labels](#labels)
    + [Functions](#functions)
      - [Set (Int, Value) -> None](#set--int--value-----none)
      - [Get (Int) -> Value](#get--int-----value)
      - [Print (Value) -> None](#print--value-----none)
      - [Write (Value) -> None](#write--value-----none)
      - [Add (Value, Value) -> Value](#add--value--value-----value)
      - [Jump (Str) -> !](#jump--str------)
      - [If (Value, Value) -> Value | !](#if--value--value-----value----)
      - [Equal (Value, Value) -> Int](#equal--value--value-----int)
      - [Extract (Str, Int) -> Str](#extract--str--int-----str)
      - [Convert (Value) -> Value](#convert--value-----value)
      - [Input () -> Str](#input-------str)
      - [KeyChar () -> Str](#keychar-------str)
      - [Number (Str) -> Int](#number--str-----int)
      - [Text (Int) -> Str](#text--int-----str)
      - [EmptySlot () -> Int](#emptyslot-------int)
      - [Exit](#exit)
      - [CatchError (Str, Value) -> Value | !](#catcherror--str--value-----value----)
      - [ThrowError (Str) -> !](#throwerror--str------)
      - [Function (Str) -> Value](#function--str-----value)
      - [Return (Value)](#return--value-)
    + [Truthy and Falsy](#truthy-and-falsy)
    + [Comments](#comments)
    + [Errors](#errors)

## Usage
To build, you need Rust installed.
```bash
git clone https://github.com/loyston500/micron-rust
cd micron-rust/micron
cargo build --release
target/release/micron # path of the executable
# to run, you simply have to pass a file
target/release/micron ../examples/mul.mc
```

## Specification
###  Data Types
1. `Int`, holds a signed integer. (isize)
2. `Str`, holds a string. (String)
3. `None`, a None type, holds null value, though not intended to be used like other two data types.

`Value` is used to indicate any data type.

### Slots
They are the variables of micron, denoted by a unique integer ranging from MIN to MAX of an isize.
By default, slots hold `None` .

### Instructions
Let's talk about the internals..
The interpreter basically iterates through every instruction (from an array) and executes them one by one.
There are two types of instructions, namely `LabelPlaceHolder` and `FunCall`.
1. `LabelPlaceHolder`, self explanatory, doesn't do much. Since labels are compile-time, the interpreter wants their locations on the array to be known.
2. `FunCall` , this is where everything happens, they are just like normal function calls you see in every other programming languages, you'll know more about them later on.

### Labels
Labels are defined using `;`
For example, `;foo`. They allow you to jump to desired locations during the execution of the program using goto-like functions.

### Functions

Just like in every programming language, they take some values, process them and give out some values.
This is true for all functions here but there are some exceptions, we'll discuss about them further down.

#### Set (Int, Value) -> None
Syntax: `s:`<br/>
This is used to set a value to the slot.
```r
s:0 20 [sets value 20 to the slot 0]
```
likewise, you can also set an Str value.
```r
s:0 "hi"
```
But the slot name should always be an Int, otherwise, it'll throw an error.
```r
s:"lol" "hi" [this will not work]
```

#### Get (Int) -> Value
Syntax: `g:`<br/>
This function returns the value of the slot. Remember that all the slots come predefined with None value, so if you try to access them, you get None.
```r
g:0 [returns the value of slot 0]
```
Examples:
 ```r
s:0 10 [sets the value of slot 0 to 10]
s:1 g:0 [gets the value of slot 0 and sets it to slot 1]
```
Since this function is going to be used a lot, you can use the short hand `.0`
which is same as `g:0`

#### Print (Value) -> None
Syntax: `p:`<br/>
Prints the given value.
```r
p:10 [this prints 10]
p:"never gonna give you up" [this prints whats expected]

s:0 "hello world"
p:.0 [prints hello world]
```

#### Write (Value) -> None
Syntax: `w:`<br/>
Same as Print but it doesn't put a new line at the end.

#### Add (Value, Value) -> Value
Syntax: `a:`<br/>
Adds two values.
If both the values are Int then you get the sum.
If both the values are Str then it will concat them and return.
Else, an error is raised.

Example:
```r
p:a: 10 20 [prints 30]
p:a: "Hello, " "World" [prints Hello, World]
p:a: 10 "uhh" [errors out]
```

####  Jump (Str) -> !
Syntax: `j:`<br/>
This one is a bit different. On calling, it jumps to the given label. Yeah, it's basically goto.
```r
p:"Hello, "
j:"foo" [jump occurs here]
p:"human" [This instruction is skipped]

;foo [it reaches here]
p:"world" [this gets printed]
```

This behaviour is consistent, regardless of it being nested or not.
```r
p:a:"hi " j:"foo"
[The evaluation of the line above is abruptly stopped and the pointer jumps to ;foo]

;foo
p:"hello" [this gets evaluated]
```

#### If (Value, Value) -> Value | !
Syntax: `?:`<br/>
If the first value is [truthy](#truthy-and-falsy), then the second value is (evaluated and) returned; If not, None is returned.

```r
p:?:10 "yay" [prints yay because 10 is truthy]
p:?:0 "not yay" [prints None because 0 is falsy]
```

Remember, the second value is evaluated ONLY if the first value is truthy. This means you can do stuff like,
```r
?:10 j:"foo" [10 is truthy so the jump function is evaluated which then performes the jump]
p:"Hello!" [this line is skipped]

;foo [that jump gets you here]
p:"Worked!" [this gets printed]
```

#### Equal (Value, Value) -> Int
Syntax: `=:`<br/>
Checks if the given values are equal, both must be of same data type, else an error is raised.
When equal, 1 (a truthy value) is returned.
When not equal, 0 (a falsy value) is returned.
`None` can't be compared, so an error is raised.

Example:
```r
p:=:10 10 [this prints 1]
p:=:20 10 [this prints 0]
p:=:"hello" "hello" [this prints 1]
p:=:"hello" "hi" [this prints 0]
p:=:"hello" 20 [this raises type error]
```

#### Extract (Str, Int) -> Str
Syntax: `x:`<br/>
Extracts a character from the Str with the given index value. If the index is out of bounds, an empty string is returned.

Example:
```r
p:x:"hello" 1 [prints e]
p:x:"hello" 100 [prints just a newline because the extract function returns an empty string]
p:x:"hello" "1" [raises error]
```

#### Convert (Value) -> Value
Syntax: `c:`<br/>
Used to convert Int to it's ASCII equivalent Str and vise versa.
Passing None or invalid values will raise `ValueError`.

Example:
```r
p:c:"f" [prints 102]
p:c:420 [prints Ƥ]
p:c:"ab" [value error is raised]
p:c:999999 [value error is raised]
```

#### Input () -> Str
Syntax: `i`<br/>
This function takes no argument so it doesn't require a colon (`:`) next to it.
Used to get user input.

Example:
```r
p:i [this prints the given input]
```

#### KeyChar () -> Str
Syntax: `k`<br/>
Same as Input function, but it doesn't need you to click enter. The given char is collected and returned.
While implementing this function, I realised that getting a key char is an OS specific thing, and apparently in some scenarios, it'll not work on terminals of Windows.  So I've decided to leave it unimplemented. It will return None instead.

#### Number (Str) -> Int
Syntax: `n:`<br/>
Converts the given Str to an Int (isize). On failure, an error is raised.

Example:
```r
p:n:"123" [prints 123]

s:1 n:"123"
p:a:.1 1 [this should print 124]

n:"foo" [raises error]
n:123 [this raises error as well but this behaviour is subjected to change]
```

#### Text (Int) -> Str
Syntax `t:`<br/>
Converts the given Int to Str. On failure, an error is raised.

Example:
```r
p:t:123 [prints 123 as usual]

s:1 t:123
p:a:.1 "1" [this should print 1231]

t:"foo" [this raises error but this behaviour is subjected to change]
```

#### EmptySlot () -> Int
Syntax: `~`<br/>
Returns the first empty slot (the slot which is set to None). Starting from 0 to MAX.

Example:
```r
p:~ [prints 0]
s:~ 10 [sets the value of slot 0 to 10]
p:~ [now it prints 1 because slot 0 is already aquired]

s:2 20 [sets the value of slot 2 to 20]
p:~ [still prints 1 because slot 1 is unused]
```

#### Exit
Syntax: `$`<br/>
Halts the entire program immediately.

Example:
```r
p:"Hi" [prints Hi]
$ [the program halts]
p:":(" [this does not print]
```

Example program that halts/exits on entering `exit`
```r
w:"Enter something: "
s:0 i
?:=:.0 "exit" $
p:"You didn't enter exit"
```

#### CatchError (Str, Value) -> Value | !
Syntax: `#:`<br/>
This is a special kind of jump function.
If an error is raised while the given value gets evaluated, it jumps to the given label while also setting the error code to slot `-1`.
Otherwise, it returns the evaluated value.

Example:
```haskell
p:#:"error" a:10 20 [prints 30 because no error occurs]
p:#:"error" a:10 "20" [error occurs so the pointer jumps to ;error]

;error
w:"Error has occured! The error code is: "
p:.-1 [-1 as described above holds the error code]
```
We'll talk about error codes later.

#### ThrowError (Str) -> !
Syntax: `!:`<br/>
Throws error with an arbitrary message.

Example:
```haskell
?:.0 !:"Slot 0 is not empty/None!" [error is not raised because None is falsy]

s:0 10
?:.0 !:"Slot 0 is not empty/None!" [now the error is raised because 10 is truthy]
```

Remember CatchError function? Yes, you can use it to catch errors thrown by this method.

```haskell
#:"error" !:"My error" [
the raised error is caught by the
catcherror function so the pointer moves to
label ;error
]

;error
w:"Got an error: "
p:.-1
```

#### Function (Str) -> Value
Syntax: `f:`<br/>
Works just like jump function, except it returns back to where it started on encountering either EOF or a return function.
On encountering EOF, the function returns None.
On encountering a return function, it essentially returns what the return function holds.

Example:
```r
w:"hello "
p:f:"return_world" [this jumps to ;return_world] [the print function receives value "world"]
$ [exit to prevent the below instructions to be executed again]

;return_world [the pointer arrives here]
r:"world" [
the return function returns the value so it
rewinds back to where the function was called
]
```

#### Return (Value)
Syntax: `r:`<br/>
 This functions returns the given value, if it's invoked by a function, then it returns it's value to it, if it's invoked during the normal execution, the program halts. The given value does get returned, you can capture it if the script itself is invoked by another script, but that feature is yet to be implemented.

```r
r:"foo"
```

### Truthy and Falsy
All numbers are truthy except for `0`.
All strings are truthy except for an empty string.
None is always falsy.

### Comments
`[` `]` are used to specify a comment. A comment can span multiple lines.
```haskell
s:10 "
hi,
I can
span multiple lines
" [
just like strings
I can also span multiple lines
]
```

### Errors
Each kind of error is associated with a unique Int.<br/>
`TypeError` => `401`,  Raised when there's a type mismatch.<br/>
`LabelError` => `402`, Raised when the label it's supposed to jump isn't defined.<br/>
`ValueError` => `403`, Raised when the value is invalid or not supported.<br/>
`NoSlotError` => `404`, Raised if there are no empty slots available from 0 to MAX.<br/>
`Error` => `400`, Raised if there's an ambiguous error.<br/>

When an error is raised, the error code is set to slot `-1`

Example:
```haskell
#:"error" j:69 [this raises TypeError since jump function always expects an Str]

;error
?:=.-1 401 j:"typeerror"
p:"Some error was raised"
$

;typeerror
p:"TypeError was raised"
```
