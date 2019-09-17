# Substrate Warmup

A Parity Substrate SRML baseline module for DockChain. The primary purpose is to work toward a core
chain and module config, which includes a multi token module and a voting module.

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

You can run the dev testnet chain with:

```bash
cargo run -- --chain=ved

# or with detailed logs
RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --chain=ved
```
