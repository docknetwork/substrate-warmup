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
cargo run --release -- --chain=ved --alice
#         ^^^^^^^^^    ^^^^^^^^^^^ ^^^^^^^
#             |             |         |
#             |             |  Use the publicly known keypair, 'Alice', to produce blocks.
#             |             |
#             |     Starting the dev chain.
#             |
# The runtime is executed purley in Wasm. The naitive runtime is disabled for this chain.
# Wasmi sometimes can't keep up with block production unless compiled with optimizations.
```
