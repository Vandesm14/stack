# Stack

[![Tests](https://github.com/Vandesm14/stack/actions/workflows/tests.yml/badge.svg)](https://github.com/Vandesm14/stack/actions/workflows/tests.yml)

<!-- [![Checks](https://github.com/Vandesm14/stack/actions/workflows/check.yml/badge.svg)](https://github.com/Vandesm14/stack/actions/workflows/check.yml) -->

An RPN stack machine built with Rust

## Installation

```bash
# After cloning the repo
cargo install --path .

# Now Stack should be installed
stack --version
```

## Usage

### Run a file

```bash
stack run <file>

# or, to watch the file for changes
stack run <file> --watch
```

### REPL

```bash
stack
```
