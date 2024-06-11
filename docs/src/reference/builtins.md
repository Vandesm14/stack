# Built-Ins

Stack comes with quite a few built-in functions (called `intrinsics` internally). They are the core of Stack and provide baseline functinoality.

### Lists vs Functions

Unless explicitly stated in the description of a function, all mentions of the type `list` includes functions (e.g.: `(fn 2 2 +)`).

## Arithmetic

**Note:** Stack uses wrapping arithmetic.

### Add (`+`)

**Signature:** `([a: int] [b: int] -- int)`

**Equivalent Rust:** `a + b`

**Examples:**
```clj
1 2 +
;; 3
```

### Subtract (`-`)

**Signature:** `([a: int] [b: int] -- int)`

**Equivalent Rust:** `a - b`

**Examples:**
```clj
2 1 -
;; 1

1 2 -
;; -1
```

### Multiply (`*`)

**Signature:** `([a: int] [b: int] -- int)`

**Equivalent Rust:** `a * b`

**Examples:**
```clj
2 3 *
;; 6
```

### Divide (`/`)

**Signature:** `([a: int] [b: int] -- int)`

**Equivalent Rust:** `a / b`

**Examples:**
```clj
6 3 /
;; 2
```

### Remainder (`%`)

**Signature:** `([a: int] [b: int] -- int)`

**Equivalent Rust:** `a & b`

**Examples:**
```clj
10 5 %
;; 0

11 5 %
;; 1
```

## Comparison

### Equal (`=`)

**Signature:** `([a] [b] -- bool)`

**Equivalent Rust:** `a == b`

**Examples:**
```clj
2 2 =
;; true

"hello" "world" =
;; false

'(1 2) '(1 2) =
;; true
```

### Not Equal (`!=`)

**Signature:** `([a] [b] -- bool)`

**Equivalent Rust:** `a != b`

**Examples:**
```clj
2 2 !=
;; false

"hello" "world" !=
;; true

'(1 2) '(1 2) !=
;; false
```

### Less Than (`<`)

**Signature:** `([a] [b] -- bool)`

**Equivalent Rust:** `a < b`

**Examples:**
```clj
1 2 <
;; true

2 1 <
;; false
```

### Less Than or Equal To (`<=`)

**Signature:** `([a] [b] -- bool)`

**Equivalent Rust:** `a <= b`

**Examples:**
```clj
1 2 <=
;; true

2 2 <=
;; true

2 1 <=
;; false
```

### Greater Than (`>`)

**Signature:** `([a] [b] -- bool)`

**Equivalent Rust:** `a > b`

**Examples:**
```clj
1 2 >
;; false

2 1 >
;; true
```

### Greater Than or Equal To (`>=`)

**Signature:** `([a] [b] -- bool)`

**Equivalent Rust:** `a >= b`

**Examples:**
```clj
1 2 >=
;; false

2 2 >=
;; true

2 1 >=
;; true
```

## Boolean

### Or (`or`)

**Signature:** `([a: bool] [b: bool] -- bool)`

**Equivalent Rust:** `a || b`

**Examples:**
```clj
false false or
;; false

false true or
;; true

true false or
;; true

true true or
;; true
```

### And (`and`)

**Signature:** `([a: bool] [b: bool] -- bool)`

**Equivalent Rust:** `a && b`

**Examples:**
```clj
false false and
;; false

false true and
;; false

true false and
;; false

true true and
;; true
```

### Not (`not`)

**Signature:** `([a: bool] -- bool)`

**Equivalent Rust:** `!a`

**Examples:**
```clj
false not
;; true

true not
;; false
```

## Stack Ops

### Drop (`drop`)

**Signature:** `([a] --)`

Drops `a` from the stack

**Examples:**
```clj
"hey"
;; ["hey"]

drop
;; []
```

### Duplicate (`dupe`)

**Signature:** `([a] -- a a)`

Duplicates `a` on the stack

**Examples:**
```clj
"hey"
;; ["hey"]

dupe
;; ["hey" "hey"]
```

### Swap (`swap`)

**Signature:** `([a] [b] -- b a)`

Swaps `a` and `b` on the stack

**Examples:**
```clj
"hello" "world"
;; ["hello" "world"]

swap
;; ["world" "hello"]
```

### Rotate (`rot`)

**Signature:** `([a] [b] [c] -- b c a)`

Rotates `a`, `b`, and `c` on the stack

**Examples:**
```clj
"a" "b" "c"
;; ["a" "b" "c"]

rot
;; ["b" "c" "a"]
```

## Lists

### Length (`len`)

**Signature:** `([a: list|string] -- int)`

**Equivalent Rust:** `a.len()`

**Examples:**
```clj
'(1 2 3) len
;; 3

"123" len
;; 3
```

### Get at Index (`nth`)

**Signature:** `([a: list] [b: int] -- a any)` or `([a: string] [b: int] -- a string)`

**Equivalent Rust:** `a[b]` or `a.get(b)`

**Examples:**
```clj
'(1 2 3) 0 nth
;; [(1 2 3) 1]

'(1 2 3) 2 nth
;; [(1 2 3) 3]

"123" 0 nth
;; ["123" "1"]

"123" 2 nth
;; ["123" "3"]
```

### Split (`split`)

**Signature:** `([a: list] [b: int] -- list list)` or `([a: string] [b: int] -- string string)`

Splits `a` at the separator `b` and returns both chunks.

**Examples:**
```clj
'(1 2 3) 1 split
;; (1) (2 3)

"123" len
;; "1" "23"
```

### Concat (`concat`)

**Signature:** `([a: list] [b: list] -- list)` or `([a: string] [b: string] -- string)`

Concats `a` and `b` together (concats the two lists or two strings)

**Examples:**
```clj
'(1) '(2 3) concat
;; (1 2 3)

"1" "23" concat
;; "123"
```

### Push (`push`)

**Signature:** `([a] [b: list] -- list)`

**Equivalent Rust:** `b.push(a)`

**Examples:**
```clj
3 '(1 2) push
;; (1 2 3)

"3" "12" len
;; "123"
```

### Pop (`pop`)

**Signature:** `([a: list] -- any)` or `([a: string] -- any)`

**Equivalent Rust:** `a.pop()`

**Examples:**
```clj
'(1 2 3) pop
;; 3

"123" len
;; "3"
```

## Records

### Insert (`insert`)

**Signature:** `([value] [key] [c: record] -- record)`

**Equivalent Rust:** `c.insert(key, value)`

**Examples:**
```clj
"value" "key" {} insert
;; {key: "value"}

true 'key {} insert
;; {key: true}

2 1 {} insert
;; {1: 2}
```

### Property (`prop`)

**Signature:** `([a: record] [b] -- a any)`

**Equivalent Rust:** `a.get(b)`

**Examples:**
```clj
{key "value"} "key" prop
;; [{key "value"} "value"]

{key "value"} "foo" prop
;; [{key "value"} nil]

{key "value"} 'key prop
;; [{key "value"} "value"]

{key "value"} 'foo prop
;; [{key "value"} nil]

{1 2} 1 prop
;; [{1 2} 2]

{1 2} 2 prop
;; [{1 2} nil]
```

### Has (`has`)

**Signature:** `([a: record] [b] -- a bool)`

**Equivalent Rust:** `a.has(b)`

**Examples:**
```clj
{key "value"} "key" has
;; [{key "value"} true]

{key "value"} "foo" has
;; [{key "value"} false]

{key "value"} 'key has
;; [{key "value"} true]

{key "value"} 'foo has
;; [{key "value"} false]

{1 2} 1 has
;; [{1 2} true]

{1 2} 2 has
;; [{1 2} false]
```

### Remove (`remove`)

**Signature:** `([a: record] [b: string] -- record)`

**Equivalent Rust:** `a.remove(b)`

**Examples:**
```clj
{key "value" foo "bar"} "foo" remove
;; [{key "value"}]

{key "value" foo "bar"} "bar" remove
;; [{key "value" foo "bar"}]
```

### Keys (`keys`)

**Signature:** `([a: record] -- a list(symbol))`

**Equivalent Rust:** `a.keys()`

**Examples:**
```clj
{key "value" foo "bar"} keys
;; [{key "value" foo "bar"} (key foo)]

{"key" "value" "foo" "bar"} keys
;; [{key "value" foo "bar"} (key foo)]
```

### Values (`values`)

**Signature:** `([a: record] -- a list)`

**Equivalent Rust:** `a.values()`

**Examples:**
```clj
{key "value" foo "bar"} values
;; [{key "value" foo "bar"} ("value" "bar")]

{f (fn 2 2 +)} values
;; [{key (fn 2 2 +)} ((fn 2 2 +))]

{f '(fn 2 2 +)} values
;; [{key '(fn 2 2 +)} ('(fn 2 2 +))]
```

## Types

### Cast (`cast`)

**Signature:** `([a] [b: string] -- any)`

Converts `a` to the type: `b` and returns the new type

### Type of (`typeof`)

**Signature:** `([a] -- string)`

Gets the type of `a` and pushes it as a string to the stack

### Lazy (`lazy`)

**Signature:** `([a] -- lazy(a))`

Wraps `a` with a lazy expression, making it lazy.

**Examples:**
```clj
1 lazy
;; '1

'()
;; ()
lazy
;; '()
```

## Control Flow

### If (`if`)

**Signature:** `([a: list] [b: bool] --)`

**Equivalent Rust:** `if b { a }`

**Examples:**
```clj
'("true")
true
if
;; "true"
```
```clj
'("true")
;; [("true")]
(4 4 =)
;; [("true") true]
if
;; ["true"]
```

### Halt (`halt`)

**Signature:** `(--)`

**Equivalent Rust:** Halts execution.

**Examples:**
```clj
2 2 halt +
;; halts before the "+"
```

### Recur (`recur`)

**Signature:** `(-- symbol)`

A QoL helper intrinsic that pushes the symbol: `recur` to the stack. Used to allow `recur` to be called without escaping with a lazy (such as `'recur`).

**Examples:**
```clj
;; Define i
0 'i def

;; Function isn't lazy so it runs right away
(fn
  ;; Our if block
  '(
    ;; Push i to the stack
    i

    ;; Add 1 to i
    i 1 + 'i set

    ;; Recur
    recur
  )

  ;; Check if i is less than 5
  i 5 <

  ;; Run the if
  if
)
;; [0 1 2 3 4]
```

### OrElse ('orelse')

**Signature:** `([a] [b] -- a|b)`

**Equivalent Rust:** `a.or(b)`

If `a` is `nil`, returns `b`. Else, returns `a`.

**Examples:**
```clj
nil 2 orelse
;; 2

1 2 orelse
;; 1
```

## Scopes and Variables

### Define (`def`)

**Signature:** `([a] [b: symbol] --)`

**Equivalent Rust:** `let b = a`

**Examples:**
```clj
0 'a def
a
;; 0

'(fn +) 'add def
2 2 add
;; 4
```

### Set (`set`)

**Signature:** `([a] [b: symbol] --)`

**Equivalent Rust:** `b = a`

**Examples:**
```clj
0 'a def
1 'a set
a
;; 1

1 'a set
;; throws since `a` is not defined
```

### Call (`call`)

**Signature:** `([a] --)`

Calls `a` and:
- If `a` is a **function**: Runs the function
- If `a` is a **list**: Runs each item in the list
- If `a` is a **symbol**: Calls the symbol from the scope
- If `a` is **anything else**: Pushes it back onto the stack

**Examples:**
```clj
2 2 
'(fn +) call
;; 4

'(2 2 +) call
;; 4

'(2 2 +) 'add def
add
;; [(2 2 +)]
call
;; [4]

'(fn +) 'add def
2 2 'add call
;; 4

0 'a def
'a call
;; 0

"foo" 'a def
'a call
;; "foo"
```

### Let (`let`)

**Signature:** `([a: list] [b: list(symbol)] --)`

Pops `b.len()` items off of the stack, assigning each item the corresponding symbol in `b`. Then, runs the code block `a`, injecting the symbols into the scope.

If list `b` was `(first second)`, then they would be popped from the stack in order, following this signature: `([first] [second] --)`.

**Examples:**
```clj
10 2 '(a b -) '(a b) let
;; 8

10 2 '(fn a b -) '(a b) let
;; 8

10 2
(fn
  '(a b -)
  '(a b)
  let
) call
;; 8
```

### Get (`get`)

**Signature:** `([a: symbol] -- any)`

**Equivalent Rust:** `a`

**Examples:**
```clj
0 'a def
'a get
;; 0

'(fn +) 'add def
2 2 add
;; 4

'(fn +) 'add def
'add get
;; (fn +)

'(fn +) 'add def
2 2 
'add get call
;; 4
```

## Debugging and I/O

### Debug (`debug`)

**Signature:** `([a] -- a)`

**Equivalent Rust:** `dbg!(format!("{}", a))`

**Examples:**
```clj
0 debug
;; prints 0

"hey" debug
;; prints "hey"
```

### Assert (`assert`)

**Signature:** `([a] [b: bool] -- a)`

**Equivalent Rust:** `assert!(b, format!("{}", a))`

**Examples:**
```clj
"my test" 2 2 = assert
;; nothing (it passes)

"my test" 1 2 = assert
;; error: assertion failed caused by my test
```

### Import (`import`)

**Signature:** `([a: string] --)`

Runs the file from path `a` in the current environment. Variables and stack changes will persist from file `a`.

**Examples:**
```clj
;; lib.stack
'(fn +) 'add def

;; main.stack
"lib.stack" import
2 2 add
;; 4
```