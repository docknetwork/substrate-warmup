# Substrate Warmup

A Parity Substrate SRML baseline module for DockChain based on the ERC20 Multi [module](https://github.com/substrate-developer-hub/substrate-erc20-multi) and [substrate node template](https://github.com/paritytech/substrate/tree/afa583011eed3e8d49ee823257a3b448a24e213b/node-template).
The primary purpose is to work toward a core chain and module config, which includes the multi token module and a voting module.

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

You can join the entnet (initial testnet) chain with:

```bash
cargo run -- --chain=entnet

# or with detailed logs
RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --chain=entnet
```
