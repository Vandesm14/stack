# Lispy Syntax

As you have seen, Stack uses postfix notation, meaning the data comes first, then the operations are applied to it. For example:

```clj
10 2 -
;; 8
```

## S-Expressions

Stack supports lisp-like (s-expressions) syntax out-of-the-box. For the example above, you could change it to:

```clj
(- 10 2)
;; 8
```

## Eager Evaluation

You can also add *most* operations within the s-expressions, which will be evaluated eagerly:

```clj
(- 10 (+ 1 1))
;; 8

(- 10 (fn (def 'a 2) a))
;; 8
```

**Important:** Functions and lists need to be lazy when used as the body of an `if` or `let`:

**`if`:**

```clj
;; As the argument: Shouldn't be lazy
(if (fn true) '("hey" print))
;; Prints "hey"

;; As the body: Should be lazy
(if true '(fn "hey" print))
;; Prints "hey"

;; Example with list as the body
(if true '["hey" print])
;; Prints "hey"

;; Example with a lazy-lazy list as the body (returns the list if true)
(if true ''["hey" print])
;; Pushes `["hey" print]` to the stack

;; Example with s-expression as the body
(if true '(print "hey"))
```

**`let`:**

```clj
;; BOTH arguments should be lazy (the symbol list and the body)

;; Lists
10 2 (let '[a b] '[a b -])
;; 8

;; Functions
10 2 (let '[a b] '(fn a b -))
;; 8

;; S-Expressions
10 2 (let '[a b] '(- a b))
;; 8
```

This is due to the eager evaluation, where both functions, lists, and s-expressions, if unlazied, will be called during eager evaluation. This can be useful for lambdas within the arguments of an s-expression

## The Underscore

To include items from the stack within an s-expression, you can use `_`. The underscore will pop the last item from the stack and use it as the argument in its place.

```clj
10 2
;; b a
(- _ _)
;; (- a b) -> (- 2 10) -> -8
```

In this example, you can see that the underscores don't order the arguments as they are visually. Instead, as the evaluator goes from left to right, they pop an item from the stack. The **first** underscore will pop the **last** item from the stack and the **second** underscore will pop the **second-to-last** item from the stack.

## Parity with Stack-based syntax

Stack's lispy syntax is one-to-one with the visual ordering of all intrinsics, except for a select few, which we will talk about in a bit.

For example, these expressions are all equivalent:

```clj
10 2 -
;; 8

(- 10 2)
;; 8

10 (- _ 2)
;; 8

2 (- 10 _)
;; 8
```

### Exceptions

For ergonomics, Stack modifies the syntax for list and record intrinsics.

**List of Exceptions:**
- `push`
- `insert`
- `let`
- `def`
- `set`

In this case, these `push` operations are equal:

```clj
0 '[] push
;; [0]

(push '[] 0)
;; [0]
```

```clj
"value" "key" {} insert
;; {key: "value"}

(insert {} "key" "value")
;; {key: "value"}
```

In these cases, the arguments for the s-expressions are reversed in comparison with the stack-based syntax. This is to aid in readability and intuition. The non-lisp syntax doesn't do this for aid in more efficient insertions or pushes:

```clj
3 2 1 '[] push push push
;; [1 2 3]

"bar" "foo" "value" "key" {} insert insert
;; {key: "value", foo: "bar"}
```

That said, pulling items from the stack works the same as with the other intrinsics:

```clj
{}
(insert _ "key" "value")
;; {key: "value"}

"key" {}
(insert _ _ "value")
;; {key: "value"}

"value" {}
(insert _ "key" _)
;; {key: "value"}

"value"
(insert {} "key" _)
;; {key: "value"}
```

Here are examples for the last few exceptions:

**Let:**

```clj
10 2 '[(- a b)] '[a b] let
;; 8

10 2 (let '[a b] '(- a b))
;; 8
```

**Def:**

```clj
0 'a def

(def 'a 0)

0 'a (def _ _)
```

**Set:**

```clj
;; define so we can set
0 'a def
;; end of boilerplate

1 'a set

(set 'a 1)

1 'a (set _ _)
```
