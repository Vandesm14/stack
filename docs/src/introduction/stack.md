# Stack

<!-- TODO: Mention auto-calling as a term as well, for reference later -->

The stack is your workspace. It works as a place to push and pop values, and to call functions. It's the main way to interact with the language.

All expressions are implicitly pushed to the stack.

<!-- TODO: Explain that symbols are caught by the engine and never pushed to the stack (unless they are lazy). Further, code and stack are separated in certain cases. -->

```clojure
2 ;; Pushes 2

"hello" ;; Pushes "hello"

'() ;; Pushes an empty list
```

## Purification

During the phase of introducing a new expression to the program, the engine "purifies" the expression. It will try to evaluate it, such as calling a symbol or evaluating a list.

```clojure
;; Push 2
2

;; Push 2
2

;; Push + (gets called automatically)
+

;; Returns 4 onto the stack
;; Both 2's and the + are popped (since + was called)
;; [2 2 +] -> [4]
```

For lists, the items are evaluated in-order, unless the list is lazy.

```clojure
(2 2 +)

;; Results in `4`
;; [] -> [4]

'(2 2 +)

;; Results in `(2 2 +)`
;; [] -> [(2 2 +)]
```

Purification only happens once. If a value from a variable is pushed to the stack, it will not be purified. The stack and scope are considered "pure". Only when putting new values into the stack will they be purified.

*(Note: Values can only get into the scope from the stack, so no purification step is made when transitioning from stack to scope, only from program/input to stack.)*

## Laziness

Because the purification step eagerly evaluates symbols and lists, this can be turned off by prefixing `'` to the beginning of an expression.

```clojure
'a

;; Results in the symbol being pushed to the stack, but not called
;; [] -> [a]
```

As a lazy expression is pushed to the stack, it will be unlazied by one level. So, if you provide two `'` in the beginning of an expression, it will be pushed as a lazy expression.

```clojure
''a

;; Results in the lazy symbol being pushed to the stack and not called
;; [] -> ['a]
```

Any expression can be made lazy, though it will really only affects symbols, lists, and functions.

```clojure
2 2 '+

;; Results in both 2's and the + being pushed to the stack, but not called
;; [] -> [2 2 +]

'(2 2 +)

;; Results in the list being pushed to the stack, but not called
;; [] -> [(2 2 +)]
```

See the docs on [lazy lists](lists.md#lazy-expressions) for more information on the behavior of lazy lists and lists with lazy items.

Calling lists, functions, and symbols will be purified then evaluated. See documentation on the [call intrinsic](../reference/builtins.md#call-call) for more information on this behavior.
