# Scopes

Stack includes a scoping mechanism that is used by functions. They are created when a new function is called. Whenever a function is pushed to the stack, its scope is scanned and set up for when it needs to run.

## Referencing

Scopes control the behavior of `def` and `set`. When a new scope is created, all variables from the previous scope will be referenced to the new scope. Any changes (via `set`) to the variables will be reflected in the previous scope.

```clojure
;; Define a variable
0 'a def

;; a == 0

;; Create a new function scope
'(fn 1 'a set) call

;; a == 1
```

## Isolation

All variables created within a function scope are isolated. Once the function has executed, its scope will be dropped, and all variables will be removed.

```clojure
'(fn
  ;; Define a variable
  0 'a def

  ;; a is defined here
  a debug  
) call

;; a is no longer defined
```

## Shadowing

Any variables created via `def` will not be reflected in the previous scope. This allows variables to be shadowed in a scope.

```clojure
;; Define a variable
0 'a def

;; a == 0

;; Create a new function scope
'(fn 1 'a def) call

;; a == 0
```

## Closures

When functions are scanned at push-time, the variables are linked together, creating a heirarchy of scopes. This allows for variables to live longer than their original scope if they are still referenced. However, this only works for **function scopes** and does not affect the symbols themselves.

An example of a closure:

```clojure
;; Create a counter function
'(fn
  ;; Define the value of the counter
  0 'value def

  ;; Return a function that increments the counter
  '(fn value 1 + 'value set value)
) 'counter def

;; Call the counter function
counter

;; Increment the counter by calling the returned function
call

;; Print the value returned from the incrementer function
debug ;; -> 1
```

In this case, the variable `value` was defined in the first function scope, but was still accessible in the second function scope. When the first function was pushed to the stack, the evaluator linked the scope of the second function to that of the first, allowing variables to be shared and reference-counted.

As long as the variable is still referenced, whether by a function on the stack or in a scope, it will stick around. Though, it will only be accessible by that function.

## Advanced

There are a few advanced and complex behaviors of Stack that allow for powerful control over scopes and metaprogramming.

### Scopeless Functions

By default, functions will have their scopes isolated. This means that any variable defined with `def` won't affect the upper scope. However, scopeless functions don't have their own scopes, and run inside the scope that they are called in.

```clojure
'(fn! 0 'a def) call

;; a == 0
```

This also works with redefining variables:
  
```clojure
;; Define a variable
0 'a def

'(fn! 1 'a def) call

;; a == 1
```

Scopeless functions allow for marcos such as importing and exporting variables into the current scope.

Simple example of this use case:

```clojure
'(fn! def) 'define def

;; Define a variable
0 'a define

;; a == 0
```

### Fn Symbols

Although `fn` looks like a symbol, it is actually a special identifier. It not only tells the evaluator to auto-call as list, but it also stores the scope data for the function. This allows for the function to be called and have its scope set up at runtime.

This means that two `fn` symbols may have different underlying metadata. So, when creating macros, it is important to keep the `fn` symbol if you are modifying existing functions:

```clojure
'(fn 0 'a def)

'(fn 1 'b def)

;; The `fn` symbols are different
```