# Built-Ins

Stack comes with quite a few built-in functions (called `intrinsics` internally). They are the core of Stack and provide baseline functinoality.

## Syntax Guide

<!-- TODO: write this -->

### Lists vs Functions

Unless explicitly stated in the description of a function, all mentions of the type `list` includes functions (e.g.: `(fn 2 2 +)`).

## Arithmetic

### Add (`+`)

**Signature:** `([a: int] [b: int] -- int)`

**Equivalent Rust:** `a + b`

### Subtract (`-`)

**Signature:** `([a: int] [b: int] -- int)`

**Equivalent Rust:** `a - b`

### Multiply (`*`)

**Signature:** `([a: int] [b: int] -- int)`

**Equivalent Rust:** `a * b`

### Divide (`/`)

**Signature:** `([a: int] [b: int] -- int)`

**Equivalent Rust:** `a / b`

### Remainder (`%`)

**Signature:** `([a: int] [b: int] -- int)`

**Equivalent Rust:** `a & b`

## Comparison

### Equal (`=`)

**Signature:** `([a] [b] -- bool)`

**Equivalent Rust:** `a == b`

### Not Equal (`!=`)

**Signature:** `([a] [b] -- bool)`

**Equivalent Rust:** `a != b`

### Less Than (`<`)

**Signature:** `([a] [b] -- bool)`

**Equivalent Rust:** `a < b`

### Less Than or Equal To (`<=`)

**Signature:** `([a] [b] -- bool)`

**Equivalent Rust:** `a <= b`

### Greater Than (`>`)

**Signature:** `([a] [b] -- bool)`

**Equivalent Rust:** `a > b`

### Greater Than or Equal To (`>=`)

**Signature:** `([a] [b] -- bool)`

**Equivalent Rust:** `a >= b`

## Boolean

### Or (`or`)

**Signature:** `([a] [b] -- bool)`

**Equivalent Rust:** `a || b`

### And (`and`)

**Signature:** `([a] [b] -- bool)`

**Equivalent Rust:** `a && b`

### Not (`not`)

**Signature:** `([a: bool] -- bool)`

**Equivalent Rust:** `!a`

## Stack Ops

### Drop (`drop`)

**Signature:** `([a] --)`

Drops `a` from the stack

### Duplicate (`dupe`)

**Signature:** `([a] -- a a)`

Duplicates `a` on the stack

### Swap (`swap`)

**Signature:** `([a] [b] -- b a)`

Swaps `a` and `b` on the stack

### Rotate (`rot`)

**Signature:** `([a] [b] [c] -- b c a)`

Rotates `a`, `b`, and `c` on the stack

## Lists

### Length (`len`)

**Signature:** `([a: list] -- int)`

**Equivalent Rust:** `a.len()`

### Get at Index (`nth`)

**Signature:** `([a: list] [b: int] -- any)`

**Equivalent Rust:** `a[b]` or `a.get(b)`

### Split (`split`)

**Signature:** `([a: list] [b: int] -- list list)` or `([a: string] [b: int] -- string string)`

Splits `a` at the separator `b` and returns both chunks.

### Concat (`concat`)

**Signature:** `([a: list] [b: list] -- list)` or `([a: string] [b: string] -- string)`

Concats `a` and `b` together (concats the two lists or two strings)

### Push (`push`)

**Signature:** `([a] [b: list] -- list)`

**Equivalent Rust:** `b.push(a)`

### Pop (`pop`)

**Signature:** `([a: list] -- any)` or `([a: string] -- any)`

**Equivalent Rust:** `a.pop()`

## Records

### Insert (`insert`)

**Signature:** `([a: string] [b: string] [c: record] -- record)`

**Equivalent Rust:** `c.insert(b, a)`

### Property (`prop`)

**Signature:** `([a: string] [b: record] -- any)`

**Equivalent Rust:** `b.get(a)`

### Has (`has`)

**Signature:** `([a: record] [b: string] -- bool)`

**Equivalent Rust:** `a.has(b)`

### Remove (`remove`)

**Signature:** `([a: record] [b: string] --)`

**Equivalent Rust:** `a.remove(b)`

### Keys (`keys`)

**Signature:** `([a: record] -- list(symbol))`

**Equivalent Rust:** `a.keys()`

### Values (`values`)

**Signature:** `([a: record] -- list(any))`

**Equivalent Rust:** `a.values()`

## Types

### Cast (`cast`)

**Signature:** `([a] [b: string] -- any)`

Converts `a` to the type: `b` and returns the new type

### Type of (`type-of`)

**Signature:** `([a] -- string)`

Gets the type of `a` and pushes it as a string to the stack

### Lazy (`lazy`)

**Signature:** `([a] -- lazy(a))`

Wraps `a` with a lazy expression, making it lazy.

## Control Flow

### If (`if`)

**Signature:** `([a: list] [b: bool] --)`

**Equivalent Rust:** `if b { a }`

### Halt (`halt`)

**Signature:** `(--)`

**Equivalent Rust:** Halts execution.

### Recur (`recur`)

**Signature:** `(-- symbol)`

A QoL helper intrinsic that pushes the symbol: `recur` to the stack. Used to allow `recur` to be called without escaping with a lazy (such as `'recur`).

## Scopes and Variables

### Call (`call`)

**Signature:** `([a] --)`

Calls `a` and:
- If `a` is a **function**: Runs the function
- If `a` is a **list**: Runs each item in the list
- If `a` is a **symbol**: Calls the symbol from the scope
- If `a` is **anything else**: Pushes it back onto the stac

### Let (`let`)

**Signature:** `([a: list] [b: list(symbol)] --)`

Pops `b.len()` items off of the stack, assigning each item the corresponding symbol in `b`. Then, runs the code block `a`, injecting the symbols into the scope.

If list `b` was `(first second)`, then they would be popped from the stack in order, following this signature: `([first] [second] --)`.

### Define (`def`)

**Signature:** `([a] [b: symbol] --)`

**Equivalent Rust:** `let b = a`

### Set (`set`)

**Signature:** `([a] [b: symbol] --)`

**Equivalent Rust:** `b = a`

### Get (`get`)

**Signature:** `([a: symbol] -- any)`

**Equivalent Rust:** `a`

## Debugging and I/O

### Debug (`debug`)

**Signature:** `([a] -- a)`

**Equivalent Rust:** `dbg!(a)`

### Assert (`assert`)

**Signature:** `([a] [b] -- a)`

**Equivalent Rust:** `assert!(b, format!("{}", a))`

### Print (`print`)

**Signature:** `([a] --)`

**Equivalent Rust:** `println!("{}", a)`

### Pretty (`pretty`)

**Signature:** `([a] --)`

**Equivalent Rust:** `println!("{:#}", val)`

<!-- TODO: OrElse -->

### Import (`import`)

**Signature:** `([a: string] --)`

Runs the file from path `a` in the current environment. Variables and stack changes will persist from file `a`.