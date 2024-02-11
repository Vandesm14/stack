# Functions

Though lists are a nice way of bundling code, they aren't perfect. For example, when creating your own named function, you will have to call it manually.

```clojure
'(1 +) 'add-one def

1 add-one
;; Pushes the list to the stack, but doesn't call it
;; [1] -> [1 (1 +)]

call
;; Calls the list
;; [1 (1 +)] -> [2]
```

## The `fn` Expression

For this reason, Stack provides the `fn` expression to mark lists as functions. Internally, what you see as a "function" is actually just a list with the `fn` expression at the beginning.

```clojure
'(fn 1 +) 'add-one def

1 add-one
;; Pushes the list to the stack and calls it
;; [1] -> [2]
```

Notice how we didn't need to call the list manually? That's because the `fn` expression tells Stack to call the list automatically. This is known as [auto-calling](/glossary.html#auto-calling).

*Note: The evaluation pattern of functions is the same as lists: left to right, evaluating each expression and pushing it to the stack.*

## Functions are Lists

Because functions are just marked lists, you can still use them as lists. This means that you can build and modify functions at runtime.

```clojure
()
'fn push
1 push
'+ push

'add-one def

1 add-one
;; Pushes the list to the stack and calls it
;; [1] -> [2]
```

*Note: The `fn` expression needs to be [lazied](/glossary.html#laziness) in order for it to not be evaluated. The evaluator **does** evaluate the `fn` expression, but it does nothing on its own (without being wrapped in a list).*

## Function Calling Behavior

When a function is pulled from scope, it will be auto-called when pushed to the stack. This is the behavior observed above, where `add-one` called the variable from scope, which Stack evaluated and called the function automatically.

*Fun Fact: All [auto-call](/glossary.html#auto-calling) does is simply append the `call` operator after the a function expression if it detects that a function is being pulled from scope and onto the stack.*

Functions can also be called manually, producing the same behavior that [auto-calling](/glossary.html#auto-calling) does.

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

## Scopeless Functions

Scopeless functions are an addition to Stack that allow for many metaprogramming aspects. Because functions have their own isolated scope, it is not possible to define variables outside of the function's scope.

Scopeless functions aren't isolated in scope and **run in the scope that they are called in**. This allows them to define or redefine variables directly in their parent scope.

```clojure
'(fn! 0 'a def) call

a

;; Pushes 0 to the stack
;; [] -> [0]
```

This is a powerful feature that allows for more dynamic programming.

In normal functions, the variable `a` would be defined in the function's scope and would not be accessible from the parent scope.

### Normal Functions

See [Scopes](/introduction/scopes.html) for more information on how scoping works and how it relates to normal functions.