# Introduction

Stack is an RPN stack-based language built in Rust. It is designed to with these goals in mind:
1. Minimal syntax (no special-cases or unknowns)
2. Code is data, data is code
3. Easy to use and understand

With these goals, Stack is designed to be a simple, yet powerful language that can be used for a variety of tasks.

## Features

Though Stack is built to be simple, it does not sacrifice power. Here are some of the features of Stack:

### Stack Ops

All functions in Stack are stack-based, meaning that they take their arguments from the stack and return their results to the stack. This allows for a simple and consistent syntax.

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

Variables are symbols, such as `my-var`. These symbols are evaluated to their value as soon as they are pushed to the stack. To prevent eager evaluation, you can make a variable "lazy" by using a `'` -> `'my-var`.

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

### Lists

Stack has a built-in list type that can be used to store multiple values. Lists are created with the `()` syntax. When pushed to the stack, the data inside lists will be evaluated.

```clojure
;; Pushes a list to the stack and evaluates it () -> ((4))
(2 2 +)
```

Just like [variables](#variables), lists can be made "lazy" by using a `'` -> `'(2 2 +)`. This will prevent the list from being evaluated when it is pushed to the stack.

```clojure
;; Pushes a list to the stack and doesn't evaluate it () -> ((2 2 +))
'(2 2 +)
```

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