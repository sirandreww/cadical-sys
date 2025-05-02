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

# Running complete CI

```
cargo clean &&
cargo fmt --all &&
git add . &&
cargo fmt --all --check &&
cargo clippy --all-targets --all-features -- -D warnings &&
cargo clippy --all-targets --all-features -- -D warnings -Dclippy::all -Dclippy::pedantic &&
cargo test --all-features --doc &&
cargo test --verbose &&
cargo publish --dry-run --allow-dirty
```

# Updating CaDiCal version

## Step 1

Update submodule
```
git submodule update --recursive --remote
```

## Step 2


1. make sure all C++ functions in `cadical/src/cadical.hpp` have interfaces in the C++ wrapper in `src/cadical_bridge.hpp`.
2. make sure all wrapped functions have unsafe rust bindings in `src/bridge.rs`
3. make sure all unsafe rust bindings have implementations in `src/lib.rs`



You can compare the API functions of the C++ CaDiCaL solver to the API of the rust CaDiCal solver like so :
```bash
cd cadical
./cnfigure
make -j
nm -C -f just-symbols --defined-only build/libcadical.a | grep CaDiCaL::Solver:: | cut -d \( -f 1 | cut -d \: -f 5 | sort > /tmp/cpp_api.txt
cd ..
cat src/lib.rs | grep pub\ fn | grep -v // | cut -d \( -f 1 | cut -d \< -f 1 | cut -d \  -f 7 |  sort > /tmp/rust_api.txt
code --diff /tmp/cpp_api.txt /tmp/rust_api.txt
```

