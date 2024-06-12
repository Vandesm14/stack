# Loops

Similar to functional programming languages such as Clojure, Stack utilizes recursion for creating loops. The engine checks if a function pushes the `recur` symbol to the stack after execution. If `recur` is detected, the engine will rerun the function, preserving the scope and using tail-call recursion.

```clojure
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

*Note: It is 100% possible to use the stack for incrementing the counter, but for the sake of readability, I used variables for this example.*
