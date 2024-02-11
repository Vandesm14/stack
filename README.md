# Stack

[![Tests](https://github.com/Vandesm14/stack/actions/workflows/tests.yml/badge.svg)](https://github.com/Vandesm14/stack/actions/workflows/tests.yml)

<!-- [![Checks](https://github.com/Vandesm14/stack/actions/workflows/check.yml/badge.svg)](https://github.com/Vandesm14/stack/actions/workflows/check.yml) -->

An RPN stack machine built with Rust

## Documentation

All of our documentation can be found in the [docs](./docs) directory or hosted via [GitHub Pages](https://vandesm14.github.io/stack/).

### Running with mdbook

To run the documentation locally, you can use `mdbook`. To install `mdbook`, you can use `cargo`:

```sh
cargo install mdbook
```

Then, to run the documentation, you can use the following command:

```sh
# Enter the docs directory
cd docs

# Run mdbook
mdbook serve --open
```