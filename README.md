# RustyLinks

RustyLinks instruments the Rust compiler, in particular the MIR (Mid-level Intermediate Representation), to do static analysis on the code. It is a research project that aims to improve the Rust programming language by providing additional information to the compiler.

## Usage

### Setup

```bash
rustup toolchain install nightly-2024-10-18
rustup component add --toolchain nightly-2024-10-18 rust-src rustc-dev llvm-tools-preview rust-analyzer clippy
```

### Test

```bash
cargo test -- --test-threads=1 --nocapture
```

### Cli (`cargo` wrapper)

> ℹ️  Additional logs can be enabled by setting the `RUST_LOG` environment variable to `debug`.

```bash
cd tests/workspaces/first
cargo run --manifest-path ../../Cargo.toml --bin cargo-rusty-links [--CARGO_ARG] -- [--PLUGIN_ARG]
```

or

```
cd tests/workspaces/first
LD_LIBRARY_PATH=$(rustc --print sysroot)/lib ../../../target/debug/cargo-rusty-links [--PLUGIN_ARG] -- [--CARGO_ARG]
```

### Driver (`rustc` wrapper)

> ⚠️  It is not currently possible to pass the plugin args to the driver without using an environment variable. Using the CLI is advised.

TODO: Find a way to pass to the driver the plugin args using "PLUGIN_ARGS" environment variable

```bash
CARGO_PRIMARY_PACKAGE=1 cargo run --bin rusty-links-driver -- ./tests/workspaces/first/src/main.rs [--RUSTC_ARG (e.g., --cfg 'feature="test"')]
```

or

```bash
cd tests/workspaces/first
CARGO_PRIMARY_PACKAGE=1 cargo run --manifest-path ../../Cargo.toml --bin rusty-links-driver -- ./src/main.rs
```
