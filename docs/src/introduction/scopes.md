# Scopes

Scopes might be the most important concept in Stack. We have put a ton of thought into how they work and how they should be used. Scopes are the backbone of Stack, and they are what make it just like any other programming language.

## What is a Scope?

Similar to other languages, a scope is a collection of variables. In Stack, a scope defines a variable as a key-value pair: symbol and expression. When a variable is defined, it is added to the current scope. When a variable is called, it is looked up in the current scope (by the [purification](/glossary.html#purification) step).

## The Main Scope

The main scope is the top-level of your program. It is the first scope that is created when you start Stack. It is the parent of all subsequent scopes. It acts as a global scope, though some behavior is isolated when inside of a function (more on that later).

In the [variable](/introduction/variables.html) section, all examples were in the main scope. Though, the behavior shouldn't change when inside of a scope other than main (such as when executing from within a function).

<!-- TODO: fact-check the statement on the behavior expectation of other scopes -->

## Creating Scopes

Scopes are only created when a function is called. When a function is called, a new scope is created for as long as the function is running. When the function is done, the scope is destroyed.

## Scoping Rules

### Access and Inheritance

This newly created scope has access to the parent scope. It can read and write to the existing variables in the parent scope. However, if a variable is defined in a child scope, is will not be accessible from within the parent scope.

In this sense, variables have full access to **existing** variables of the parent scope, but cannot introduce **new** variables into that scope.

### Shadowing

When a variable is defined in a child scope with the same name as a variable in the parent scope, the child scope's variable will "shadow" the parent scope's variable. This means that the child scope's variable will be used instead of the parent scope's variable.

To the function, it will have access to its own variable while the parent scope's variable is still accessible from within the parent scope.