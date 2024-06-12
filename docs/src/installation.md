# Installation

Stack is not currently in the cargo registry, so to install, you must clone the repo and install it manually with `cargo`.

```bash
# Clone the repo
git clone https://github.com/vandesm14/stack

# Move into the directory
cd stack

# After cloning the repo
cargo install --path stack-cli

# Now Stack should be installed
stack --version
```

## Usage

Use `stack --help` for further documentation.

### REPL

You can also use the REPL to run code interactively.

```bash
stack repl
```

### Run a file

To run a file, use the `run` subcommand.

```bash
stack run <file>

# or, to watch the file for changes
stack run --watch <file>
```
