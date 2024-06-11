# Lists

Lists are similar to arrays or vectors in other programming languages. A list can contain any number of items of any type.

## Defining a List

Lists are defined using the `'()` syntax. The items inside of a list are separated by spaces.

```clojure
'(1 2 3 4 5)
```

## Eager Evaluation

When lists are pushed to the stack, the items inside of the list are evaluated in-order (due to the [purification](../introduction/stack#purification) step).

```clojure
(1 2 3)

;; Results in `1 2 3`
;; [] -> [(1 2 3)] -> [1 2 3]

(2 2 +)

;; Results in `4`
;; [] -> [(2 2 +)] -> [4]

2 'var def

(var)

;; Results in `2`
;; [] -> [(var)] -> [2]
```

### Laziness

Symbols (variables) inside of lazy lists will not be evaluated.

```clojure
2 'var def

'(var)

;; Results in `(var)`
;; [] -> [(var)]
```

Instead, to add variables into a list, it will need to be created manually.

```clojure
2 'var def

var '() push

;; Results in `(2)`
;; [] -> [(2)]
```

You can make specific items inside of a list lazy by prefixing `'` to the beginning of the item.

```clojure
(2 2 '+)

;; Results in the items being pushed to the stack, but the `+` will not be not called
;; [] -> [2 2 +]

(2 2 + 5 '*)

;; Results in the items being pushed to the stack, and only the `*` will not be called
;; [] -> [4 5 *]
```

## Calling Lists

Lazy lists can be called which will run each expression in the list in-order (left to right). This allows code to be bundled in a list, and evaluated later.

Calling a list exhibits the same behavior as the [purification](../introduction/stack.md#purification) step.

```clojure
'(2 2 +) call

;; Results in `4` being pushed to the stack
;; [] -> [2 2 +] -> [4]
```

If a list contains callable items such as other lists, those will also be run.

```clojure
'((2 2 +)) call

;; Results in `4` being pushed to the stack
;; [] -> [2 2 +] -> [4]
```

To change this behavior, any callable items should be made lazy when adding them to the list to ensure that they won't be called.

```clojure
'(2 2 +)
;; (2 2 +)
lazy
;; '(2 2 +)
'() push
;; ('(2 2 +))
call

;; Results in `(2 2 +)` being pushed to the stack
;; [] -> [(2 2 +)] -> [(2 2 +)]
```

<!-- **Note: Running `call` on a list doesn't provide the same behavior as the [purification](../introduction/stack#purification) step. It evaluates the items in the list, and doesn't keep the items inside the bounds of the list. To keep the items inside the bounds of the list, you can use the `call-list` operator.** -->

<!-- TODO: we need to add the call-list intrinsic -->
<!-- ## The `call-list` Operator

To perform the same behavior as pushing a non-lazy list to the stack, to a lazy list, you can use the `call-list` operator. This works differently than `call`, which evaluates and unwraps the results onto the stack.

```clojure
'(2 2 +) call-list

;; Results in `(4)` being pushed to the stack
;; [] -> [(4)]
``` -->
