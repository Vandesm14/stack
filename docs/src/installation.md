# Installation

Stack is currently not in the cargo registry, so to install, you must clone the repo and install it manually.

```bash
# Clone the repo
git clone https://github.com/vandesm14/stack

# Move into the directory
cd stack

# After cloning the repo
cargo install --path .

# Now Stack should be installed
stack --version
```

## Usage

### REPL

You can also use the REPL to run code interactively.

```bash
stack
```

### Run a file

To run a file, use the `run` subcommand.

```bash
stack run <file>

# or, to watch the file for changes
stack run <file> --watch
```