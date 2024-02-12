# Syntax

In Stack, there are a few basic types of data, and a few basic ways to structure them. Here are some examples:

<!-- TODO: Show the complete list of allowed chars in symbols -->
<!-- TODO: Double-check that this is everything -->

```clojure
;; Integers
1 -1

;; Floats
1.0 -1.0

;; Strings
"Hello, World!" "Hello, \"World!\""

;; Booleans
true false

;; Symbols (aka "Calls")
+ - * / % = < > <= >= != my-symbol what/is_this-madness?!

;; Lists
(1 2 3) (('a 'pair) ('of 'items))

;; Lazy
'my-symbol '() 'fn

;; Functions
'(fn 2 2 +)

;; Comments
;; This is a comment

;; Do-Nothing brackets (they do nothing! good for visual structuring.)
[0 'a def] [a 2 +]
```

All whitespace is treated the same, so you have really long one-liners or split each item onto its own line. It's up to you!