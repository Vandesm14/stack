# Lists

Lists are similar to arrays or vectors in other programming languages. A list can contain any number of items of any type.

## Defining a List

Lists are defined using the `()` syntax. The items inside of a list are separated by spaces.

```clojure
(1 2 3 4 5)
```

## Eager Evaluation

When lists are pushed to the stack, the items inside of the list are evaluated in-order and kept inside the bounds of the list (due to the [purification](../introduction/stack#purification) step).

```clojure
(2 2 +)

;; Results in `(4)`
;; [] -> [(2 2 +)] -> [(4)]
```

You can also use symbols (variables) in non-lazy lists, which will evaluate to their values.

```clojure
2 'var def

(var)

;; Results in `(2)`
;; [] -> [(var)] -> [(2)]
```

## Laziness

### Lazy Lists

To avoid eager evaluation, you can make a list lazy by prefixing `'` to the beginning of the list.

```clojure
'(2 2 +)

;; Results in the list being pushed to the stack, but not called
;; [] -> [(2 2 +)]
```

### Lazy Expressions

Alternatively, you can make specific items inside of a list lazy by prefixing `'` to the beginning of the item.

```clojure
(2 2 + 5 '*)

;; Results in the list being pushed to the stack, and partially called
;; [] -> [(4 5 *)]
```

## Calling Lists

Lists can also be called, though they have a different behavior than calling symbols. When called, each expression a list will be evaluated in order (left to right). This allows code to be bundled in a list, and evaluated later.

```clojure
'(2 2 +) call

;; Results in `4` being pushed to the stack
;; [] -> [2 2 +] -> [4]
```

Notice that the list was made lazy using the `'` prefix so we can call it manually.

**Note: Running `call` on a list doesn't provide the same behavior as the [purification](../introduction/stack#purification) step. It evaluates the items in the list, and doesn't keep the items inside the bounds of the list. To keep the items inside the bounds of the list, you can use the `call-list` operator.**

## The `call-list` Operator

To perform the same behavior as pushing a non-lazy list to the stack, to a lazy list, you can use the `call-list` operator. This works differently than `call`, which evaluates and unwraps the results onto the stack.

```clojure
'(2 2 +) call-list

;; Results in `(4)` being pushed to the stack
;; [] -> [(4)]
```