# Introduction

Stack is an RPN stack-based language built in Rust. It is designed to be simple and easy to learn, while still being powerful and flexible. Stack is built with the hard decisions done in the evaluator, allowing you to do almost anything in-language. Things such as imports, modules, and even the standard library are almost entirely written in Stack.

## Features

Though Stack is built to be simple at the top-level, it does not sacrifice power. Here is a quick introduction to the base features of the language:

### Stack Ops

All functions in Stack are built for the stack, meaning that they take their arguments from the stack and return their results to the stack. This allows for a simple and consistent syntax.

```clojure
;; Push 2 to the stack () -> (2)
2

;; Duplicate the item on the stack (2) -> (2 2)
dup

;; Add the top two items on the stack (2 2) -> (4)
+

;; Print the top item on the stack (4) -> ()
debug
```

### Variables

Variables are represented as symbols, such as `my-var`. These symbols are evaluated to their value as soon as they are pushed to the stack. To prevent eager evaluation, you can make a variable "lazy" by using a `'` -> `'my-var`. Lazy symbols aren't evaluated and are pushed to the stack as raw symbols. This is useful for macros and other metaprogramming features.

```clojure
;; Push 0 to the stack () -> (0)
0

;; Push `my-var` to the stack and don't evaluate it (0) -> (0 my-var)
'my-var

;; Push `def` to the stack and evaluate it (0 my-var) -> ()
def

;; Now, `my-var` is defined as 0
my-var debug ;; prints 0
```

Variables work like so:
- `def` is used to define a variable and assign a value to it
  - This can be used to set variables as well, though it will do so within the current scope
- `set` is used to change the value of an existing variable
  - This will error if you try to set a variable that doesn't exist

### Lists

Stack has a built-in list type that can be used to store multiple values. Lists are created with the `()` syntax. When pushed to the stack, the data inside lists **will be evaluated**. The evaluation happens from left to right, and the result is reimplemented into the list. For example, the list below adds two numbers and is collapsed down into the result.

```clojure
;; Pushes a list to the stack and evaluates it () -> ((4))
(2 2 +)
```

However, you can also reference variables in lists and they will be evaluated when the list is pushed to the stack.

```clojure
;; `my-var` is defined as 0
0 'my-var def

;; Pushes a list to the stack and evaluates it () -> ((0))
(my-var)
```

The symbol `my-var` in the list will be evaluated to its value, 0, when the list is pushed to the stack.

#### Laziness

Just like [variables](#variables), lists can be made "lazy" by using a `'` -> `'(2 2 +)`. This will prevent the list from being evaluated when it is pushed to the stack.

```clojure
;; Pushes a list to the stack and doesn't evaluate it () -> ((2 2 +))
'(2 2 +)
```

In this case, all items in the list are kept as they are.

Alternatively, you can make only certain items in a list lazy `'`, like so:

```clojure
;; `my-var` is defined as 0
0 'my-var def

;; Pushes a list to the stack and doesn't evaluate the first item () -> ((my-var 0))
('my-var my-var)
```

In this case, the first item in the list is kept as a lazy symbol, while the second item is evaluated.

### Functions

Functions are the primary way to define new operations in Stack. Uniquely, Stack doesn't have a special data type for functions. Instead, functions are just lists of operations. Heres what they look like:

```clojure
;; Push a function (a lazy list) to the stack () -> ('(...))
'(fn 1 +)

;; Push a lazy symbol to name the function ('(...)) -> ('(...) 'add-one)
'add-one

;; Push and evaluate the `def` symbol ('(...) 'add-one) -> ()
def

;; Push 0 to the stack () -> (0)
0

;; Push `add-one` to the stack and evaluate it (0) -> (1)
add-one
```

The `fn` symbol at the beginning of the list tells the evaluator to treat the list as a function. If this was ommitted, each instruction on the list would be evaluated separately.

```clojure
;; Push a lazy list to the stack () -> ('(1 +))
'(1 +)

;; Push a lazy symbol to name the function ('(1 +)) -> ('(1 +) 'add-one)
'add-one

;; Push and evaluate the `def` symbol ('(1 +) 'add-one) -> ()
def

;; Push 0 to the stack () -> (0)
0

;; Push the value of `add-one` to the stack (0) -> (0 (1 +))
add-one

;; Evaluate the code in the list (0 (1 +)) -> (0)
call
```

This is known as *auto-call*, where `fn` at the beginning of the list tells the evaluator to push `call` to the stack after the list, which will evaluate the list.

#### Scoping

Scopes are handled by the evaluator and are not built into the language. Read more in the [Scopes](features/scopes.md) section.

### Brackets

A small quality-of-life feature of Stack is the inclusion of `[]` that... do nothing! Because Stack doesn't require parenthesis for function calls, you can use brackets to group code without changing the behavior of the program.

```clojure
[0 'my-var def]
[my-var debug]

[my-var] [2 +] ['my-var set]
[my-var debug]
```

### Macros

Macros are a powerful feature of Stack that allow you to modify code at runtime. Uniquely, macros are just functions. This allows you to use the full power of the language to modify code, without special-casing behavior for macros.

```clojure
;; Define a macro that adds a suffix to a symbol
'(fn
  ;; Turn the symbol into a string
  tostring
  
  ;; Wrap the string into a list (string) -> ((string))
  wrap
  
  ;; Wrap the suffix into a list
  "suffix" wrap
  
  ;; Concatenate the two lists
  concat
  
  ;; Join the list into a string, separated by "/"
  "/" join
  
  ;; Turn the string into a symbol
  tocall
) 'wrap-suffix def

;; Turns `symbol` into `symbol/suffix`
'symbol wrap-suffix
```