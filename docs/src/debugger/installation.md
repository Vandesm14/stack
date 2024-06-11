# Installation

Stack comes with a debugger that provides a GUI to aid in debugging and inspecting the behavior of Stack.

```bash
# Clone the repo
git clone https://github.com/vandesm14/stack

# Move into the directory
cd stack

# After cloning the repo
cargo install --path stack-debugger

# Now the debugger should be installed
stack-debugger --version
```

## Usage

Use `stack-debugger --help` for further documentation.

### Run a file

To debug a file, provide it with a path.

```bash
stack-debugger <file>
```

The debugger automatically watches the file for changes and reruns the code.