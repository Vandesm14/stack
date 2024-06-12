# Stack

[![Tests](https://github.com/Vandesm14/stack/actions/workflows/tests.yml/badge.svg)](https://github.com/Vandesm14/stack/actions/workflows/tests.yml)
[![Docs](https://github.com/Vandesm14/stack/actions/workflows/deploy_book.yml/badge.svg?branch=main)](https://vandesm14.github.io/stack/)

Stack is a dynamic, stack-based, [concatenative] programming language.

## Roadmap

We have a public roadmap hosted on [Trello](https://trello.com/b/xEZN90Zx/stack).

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

## Licence

All code in this repository is dual-licensed under, at your option, either:

- MIT Licence ([LICENCE-MIT](./LICENCE-MIT) or http://opensource.org/licenses/MIT); or
- Apache 2.0 Licence ([LICENCE-APACHE](./LICENCE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0).

Any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache 2.0 Licence, shall be dual-licensed as above, without any additional terms or conditions.

[concatenative]: https://concatenative.org/
