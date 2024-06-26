# Functions

Though lists are a nice way of bundling code, they aren't perfect. For example, when creating your own named function, you will have to call it manually.

```clojure
'[1 +] 'add-one def

1 add-one
;; Pushes the list to the stack, but doesn't call it
;; [1] -> [1 [1 +]]

call
;; Calls the list
;; [1 [1 +]] -> [2]
```

## The `fn` Expression

For this reason, Stack provides parenthetical syntax to create functions. They function similar to lists, except they start with a `fn` or `fn!` identifier and use parenthesis instead of square brackets.

```clojure
'(fn 1 +) 'add-one def

1 add-one
;; Pushes the list to the stack and calls it
;; [1] -> [2]
```

Notice how we didn't need to call the list manually? That's because the `fn` expression tells Stack to call the list automatically. This is known as **auto-calling**.

*Note: The evaluation pattern of functions is the same as lists: left to right, evaluating each expression and pushing it to the stack.*

## Functions are Lists

Though functions are different from lists, you can still use most of the list methods on them (more info [here](../reference/builtins.md)). This means that you can build and modify functions at runtime.

```clojure
'(fn)
1 push
'+ push
;; [] -> [(fn 1 +)]

'add-one def

1 add-one
;; Pushes the list to the stack and calls it
;; [1] -> [2]
```

<!-- TODO: explain how the `fn` symbol will inherit the scope from your function. A solution for macros is to bring in a `(fn)` instead of building it yourself -->

*Note: The `fn` expression needs to be made lazy (with `'` -> `'fn`) in order for it to not be evaluated. The engine does evaluate the `fn` expression, but it does nothing on its own (without being wrapped in a list).*

## Function Calling Behavior

When a function is pulled from scope, it will be auto-called when pushed to the stack. This is the behavior observed above, where `add-one` called the variable from scope, which Stack evaluated and called the function automatically.

Functions can also be called manually, producing the same behavior that **auto-calling** does.

**For anonymous functions:**
```clojure
'(fn 2 2 +) call

;; Pushes 4 to the stack
;; [] -> [4]
```

**For named functions**
```clojure
'(fn 2 2 +) 'add-two def

'add-two get
;; Pushes the list to the stack
;; [] -> [(fn 2 2 +)]

call
;; Calls the list
;; [(fn 2 2 +)] -> [4]
```

### The `get` Operator

To get the function itself from the scope, to bypass auto-calling, you can use the `get` operator.

```clojure
'(fn a) 'my-fn def

'my-fn get

;; Results in (fn a) being pushed to the stack
;; [] -> [(fn a)]
```

## Scopeless Functions

Normal functions have their own isolated scope, it is not possible to define variables outside of the function's scope. Because of this, Stack includes a "mode" of function called a **scopeless function**. This is a powerful feature that enables for more dynamic meta-programming.

Scopeless functions don't have their own isolated scope and **run in the scope that they are called in**. This allows them to define or redefine variables directly in the scope that they were called in.

```clojure
'(fn! 0 'a def) call

a

;; Pushes 0 to the stack
;; [] -> [0]
```

```clojure
;; Create a scopeless function
'(fn! 0 'a def)

;; Create a normal function that calls the scopeless function
'(fn
  call

  ;; `a` is 0
)

;; Call the normal function
call

;; `a` doesn't exist here, since it was part of the previous function's scope
```

See [Scopes](../introduction/scopes) for more information on how scoping works and how it relates to normal functions.
