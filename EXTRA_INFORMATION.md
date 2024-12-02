# Generating README

We  use a cargo package to make the readme from the docs, to update the readme run the following command:

```
cargo readme > README.md
```

For more information: https://crates.io/crates/cargo-readme

# Viewing the DOCS

To view the documentation for this library, rust provides a way to compile comments into docs:

```
cargo doc --open
```

# Running Valgrind

To run valgrind run:

```
cargo valgrind test --verbose
```