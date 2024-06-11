# Scopes

<!-- TODO: Maybe prefer referencing scopes as outer and inner -->

Scopes are one of the most important concepts in Stack as they set Stack apart from traditional concatenative languages. We have put a ton of thought into how they work and how they should behave.

## What is a Scope?

Similar to other languages, a scope is a collection of symbol-expression pairs. When a variable is defined, it is added to the current scope. When a variable is called, it is looked up in the current scope (by the [purification](stack.md#purification) step).

## The Main Scope

The main scope is the top-level of your program. It is the first scope that is created when you start Stack. It is the outermost scope and acts as a global scope.

In the [variable](variables.md) section, all examples were in the main scope. Though, the behavior of those examples doesn't change when inside of a scope other than main (such as when within a function).

## Creating Scopes

When a function is called, a new scope is created that lives for as long as the function is running. When the function is done, the scope is destroyed.

## Scoping Rules

### Isolation

Inner scopes are isolated from their outer scope. This means that variables defined in the inner scope are not accessible from the outer scope and will be destroyed when the inner scope is destroyed (unless they are referenced, see [closures](#closures)).

<!-- TODO: not unless they're referenced, it's whenever an inner scope exists within the outer scope, that outer scope is copied to the inner and will live as long as the inner scope (thanks Rust Rc's). -->

```clojure
'(fn 0 'a def) call

a
;; Throws an error because `a` is not defined in the main scope
```

### Access and Inheritance

This newly created scope has access to the outer scope. It can read and write to the existing variables in the outer scope. However, if a variable is defined in a inner scope, is will not be accessible from the outer scope.

Therefore, inner scopes have full access to **existing** variables of the outer scope, but cannot introduce **new** variables into that (outer) scope.

**Getting:**

```clojure
0 'a def

'(fn a) call

;; Pushes 0 to the stack
;; [] -> [0]
```

**Setting:**

```clojure
0 'a def

'(fn 1 'a set) call

a
;; Pushes 1 to the stack
;; [] -> [1]
```

### Shadowing

When a variable is defined in a inner scope with the same name as a variable in the outer scope, the inner scope's variable will "shadow" the outer scope's variable. This means that the inner scope's variable will be used instead of the outer scope's variable. So, the inner scope has access to its own variable while the outer scope's variable is still accessible from within the outer scope.

```clojure
0 'a def

'(fn 1 'a def a) call
;; Pushes 1 to the stack
;; [] -> [1]

a
;; Pushes 0 to the stack
;; [] -> [0]
```

### Closures

When a variable is referenced in a inner scope, it will be kept alive in the outer scope. This is called a closure and is similar to the behavior in languages such as JavaScript.

```clojure
;; This function isn't lazy, so it will run automatically
(fn
  0 'a def

  '(fn a)
)
;; (returns the inner function)
;; Pushes `(fn a)` to the stack
;; [] -> [(fn a)]

;; Call the inner function
call
;; Pushes `0` to the stack
;; [(fn a)] -> [0]
```

As you can see, the inner function still has access to the outer scope's variable `a` even though the outer function finished executing.

<!-- TODO: Rewrite this as it doesn't matter if a symbol is referenced, the outer scope will always exist for the inner scope -->
<!-- ## Scope Implementation

Closures are enabled by an internal scanning system. The scanner is triggered whenever a function is pushed to the stack and recursively scans all symbols that the function contains. However, the scanner is dumb and doesn't know how a function will use its symbols. Because of this, the scanner makes decisions based on whether a symbol in the function exists in the outer scope.

- If the outer scope has the symbol, the symbol's value is referenced in the function's scope.
- If the outer scope does not have the symbol, the symbol is reserved in the function's scope (defined with no value, erroring if called before defining).

When a function is called, the scope of the function is set as the current scope. The function can shadow values in its scope, which will propagate to any inner functions.

### Closures with State

This example is the simplest. The scanner sees `a` being referenced in the function, and it links the scope entry for `a` in the inner function to the outer scope. The inner function could always redefine (shadowing), which would unlink the entry.

```clojure
'(fn
  0 'a def

  '(fn a)
)

;; Call the outer function
;; (returns the inner function)
call

;; Call the inner function
call
;; Pushes 0 to the stack
;; [] -> [0]
```

### Rescanning

Still referencing the above example, the scanner initially scans the outer function recursively, including the inner function, when the outer is pushed to the stack. Then, after the outer function runs, it pushes the inner function to the stack. The scanner is triggered from that push, and rescans the inner function.

This is important because it means the scanner doesn't need to rely on symbols being used in upper scopes, since it will inherit the symbols from the outer scope (which would be defined at runtime).

Here is a simple example of this behavior.

```clojure
;; Push the symbol `a`, then `0`, then `a` again
'a 0 'a

;; Pushes the outer function (triggering a scan of both functions)
'(fn
  ;; Pulls the `a` and `0` from the stack, defining it in scope
  def

  ;; Pushes the inner function (triggering another scan)
  ;; that uses the last `a` on the stack to get the value
  '(fn get)
)

;; Call both functions
call call

;; Pushes 0 to the stack
;; [] -> [0]
```

In this example, we are pushing a pattern that `def` can use (`value`, `symbol`) and also including another symbol for the inner function. The initial scan of the outer function wouldn't see any symbols being used, thus leaving a blank scope for both outer and inner functions.

Once the outer function is called, it will push the inner function to the stack, triggering a rescan of the inner function. This time, `a` will be in the current scope of the scanner, and the inner function will inherit the value of `a` from the outer scope.

Thus, the scanner doesn't need to "see" the variables in the code itself, it just needs to have the variables ready by the time it scans (or rescans) a function. -->
