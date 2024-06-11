# Variables

Stack includes variables in conjunction with the symbol system. When a symbol is pushed to the stack, the evaluator will try to call the respective variable or native function.

Native functions, such as `+` don't exist in the scope, but are instead native Rust code that is built with the evaluator. Variables, however, are stored in the scope and only exist for the evaluation instance.

It is not possible to redefine native functions, but it is possible to redefine variables.

## Defining Variables

Variables are defined using the `def` operator. The first argument is the name of the variable, and the second is the value.

```clojure
0 'a def
```

Notice how `a` is made lazy with the `'` prefix. This prevents `a` from being called and is placed on the stack as a raw symbol.

```clojure
'a

;; Results in `a` being pushed to the stack, but not called
;; [] -> [a]
```

## Using Variables

### Symbol Calls

You can get the value of a variable by pushing the respective symbol (name of the variable) to the stack.

```clojure
0 'a def

;; Push the symbol `a` to the stack, which gets called automatically
a

;; Results in `0` being pushed to the stack
;; [] -> [0]
```

Remember, we don't need to make the second `a` lazy, because we want it to evaluate to the value.

## Updating Variables

Variables can be updated using the `set` operator. The first argument is the name of the variable, and the second is the new value.

```clojure
0 'a def

1 'a set

a

;; Results in `1` being pushed to the stack
;; [] -> [1]
```

If you try to update a variable that doesn't exist, it will result in an error.

## Deleting Variables

Variables can be deleted using the `undef` operator. The first argument is the name of the variable.

```clojure
0 'a def

'a undef

a

;; Results in an error because `a` is no longer defined
;; [] -> []
```
