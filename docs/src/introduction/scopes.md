# Scopes

Scopes might be the most important concept in Stack. We have put a ton of thought into how they work and how they should be used. Scopes are the backbone of Stack, and they are akin to applicative languages that most people are familiar with.

## What is a Scope?

Similar to other languages, a scope is a collection of symbol-expression pairs. When a variable is defined, it is added to the current scope. When a variable is called, it is looked up in the current scope (by the [purification](/glossary.html#purification) step).

## The Main Scope

The main scope is the top-level of your program. It is the first scope that is created when you start Stack. It is the parent of all subsequent scopes. It acts as a global scope, though some behavior is isolated when inside of a function (more on that later).

In the [variable](/introduction/variables.html) section, all examples were in the main scope. Though, the behavior shouldn't change when inside of a scope other than main (such as when executing from within a function).

## Creating Scopes

When a function is called, a new scope is created for as long as the function is running. When the function is done, the scope is destroyed.

## Scoping Rules

### Isolation

Child scopes are isolated from the parent scope. This means that variables defined in the child scope are not accessible from the parent scope and will be destroyed when the child scope is destroyed (unless they are referenced, see [closures](#closures)).

```clojure
'(fn 0 'a def) call

a
;; Throws an error because `a` is not defined in the main scope
```

### Access and Inheritance

This newly created scope has access to the parent scope. It can read and write to the existing variables in the parent scope. However, if a variable is defined in a child scope, is will not be accessible from within the parent scope.

Therefore, child scopes have full access to **existing** variables of the parent scope, but cannot introduce **new** variables into that (parent) scope.

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

When a variable is defined in a child scope with the same name as a variable in the parent scope, the child scope's variable will "shadow" the parent scope's variable. This means that the child scope's variable will be used instead of the parent scope's variable. So, the child scope has access to its own variable while the parent scope's variable is still accessible from within the parent scope.

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

When a variable is referenced in a child scope, it will be kept alive in the parent scope. This is called a closure. This is a powerful feature that allows for more dynamic programming.

```clojure
'(fn
  0 'a def

  '(fn a)
)

;; Call the parent function
;; (returns the child function)
call

;; Call the child function
call
;; Pushes 0 to the stack
;; [] -> [0]
```

As you can see, the child function still has access to the parent scope's variable `a` even though the parent function finished executing.

#### Implementation

Closures are enabled by an internal scanning system. The scanner is triggered whenever a function is pushed to the stack and recursively scans all symbols that the function contains. However, the scanner is dumb and doesn't know how a function will use its symbols. Because of this, the scanner makes decisions based on whether a symbol in the function exists in the parent scope.

- If the parent scope has the symbol, the symbol's value is referenced in the function's scope.
- If the parent scope does not have the symbol, the symbol is reserved in the function's scope (defined with no value, erroring if called before defining).

When a function is called, the scope of the function is set as the current scope. The function can shadow values in its scope, which will propagate to any child functions.

**Closures with State:**

This example is the simplest. The scanner sees `a` being referenced in the function, and it links the scope entry for `a` in the child function to the parent scope. The child function could always redefine (shadowing), which would unlink the entry.

```clojure
'(fn
  0 'a def

  '(fn a)
)

;; Call the parent function
;; (returns the child function)
call

;; Call the child function
call
;; Pushes 0 to the stack
;; [] -> [0]
```

**Rescanning:**

Still referencing the above example, the scanner initially scans the parent function recursively, including the child function, when the parent is pushed to the stack. Then, after the parent function runs, it pushes the child function to the stack. The scanner is triggered from that push, and rescans the child function.

This is important because it means the scanner doesn't need to rely on symbols being used in upper scopes, since it will inherit the symbols from the parent scope (which would be defined at runtime).

Here is a simple example of this behavior.

```clojure
;; Push the symbol `a`, then `0`, then `a` again
'a 0 'a

;; Pushes the parent function (triggering a scan of both functions)
'(fn
  ;; Pulls the `a` and `0` from the stack, defining it in scope
  def

  ;; Pushes the child function (triggering another scan)
  ;; that uses the last `a` on the stack to get the value
  '(fn get)
)

;; Call both functions
call call

;; Pushes 0 to the stack
;; [] -> [0]
```

In this example, we are pushing a pattern that `def` can use (`value`, `symbol`) and also including another symbol for the child function. The initial scan of the parent function wouldn't see any symbols being used, thus leaving a blank scope for both parent and child functions.

Once the parent function is called, it will push the child function to the stack, triggering a rescan of the child function. This time, `a` will be in the current scope of the scanner, and the child function will inherit the value of `a` from the parent scope.

Thus, the scanner doesn't need to "see" the variables in the code itself, it just needs to have the variables ready by the time it scans (or rescans) a function.