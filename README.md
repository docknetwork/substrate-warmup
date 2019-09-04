# Substrate Warmup

SRML-based Substrate node derived from [substrate node template](https://github.com/paritytech/substrate/tree/afa583011eed3e8d49ee823257a3b448a24e213b/node-template).

# Requires

```bash
# Rust
curl https://sh.rustup.rs -sSf | sh

# wasm32-unknown-unknown target
rustup target add wasm32-unknown-unknown --toolchain nightly

# wasm-gc
cargo +nightly install --git https://github.com/alexcrichton/wasm-gc --force
```

# Run

You can start a development chain with:

```bash
cargo run -- --dev

# or with detailed logs
RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev

# or get available command line options
cargo run -- --help
```
