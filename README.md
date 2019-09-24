# Substrate Warmup

A Parity Substrate SRML baseline module for DockChain. The primary purpose is to work toward a core
chain and module config, which includes a multi token module and a voting module.

# Quick setup using Docker

```
docker build -t full-node .
docker run -p 9944:9944 full-node --alice
```

See the Dockerfile for more details.

# Development setup

## Requires

```bash
# Rust
curl https://sh.rustup.rs -sSf | sh

# Nightly toolchain for building runtime
rustup toolchain add nightly

# wasm32-unknown-unknown target
rustup target add wasm32-unknown-unknown --toolchain nightly

# wasm-gc
cargo +nightly install --git https://github.com/alexcrichton/wasm-gc --force

# substrate
git clone https://github.com/paritytech/substrate.git
cargo install --path substrate --force
```

## Run

You can run the dev testnet chain with:

```bash
# make a temporary directory in which to store chain data
mkdir -p tmp

# create a chainspec
cargo run --release -- ved > tmp/chainspec.json
#         ^^^^^^^^^    ^^^ ^^^^^^^^^^^^^^^^^^^^
#             |         |         |
#             |         | Dump the chainspec into a file which we'll use in the next step.
#             |         |
#             | Specify the dev chain.
#             |
# The runtime is executed purley in Wasm. The naitive runtime is disabled for this chain.
# Wasmi sometimes can't keep up with block production unless compiled with optimizations.
# In addition to the being slow, the runtime much larger when compiled without --release.

# run created chainspec using substrate
substrate --chain ./tmp/chainspec.json --alice --base-path ./tmp
#         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ^^^^^^^ ^^^^^^^^^^^^^^^^^
#             |                           |            |
#             |                           | Store chain data in a temporary directory.
#             |                           |
#             | Use the publicly known keypair, 'Alice', to produce blocks.
#             |
# Run the chain specification we specified in the previous command.
```

# Using the polkadot js UI

Once the dev chain is running, you can interact with it via browser.

Go to https://polkadot.js.org/apps/#/settings and set "remote endpoint" to your locally running node 127.0.0.1.

Copy the type definitions in "./ui-types.json" to https://polkadot.js.org/apps/#/settings/developer and save.

To interact with the dev chain, you'll need to load the private key for the superuser, Alice, into the browser ui.
Go to https://polkadot.js.org/apps/#/accounts and add an account with the publicly known seed
`bottom drive obey lake curtain smoke basket hold race lonely fit walk`. Derive the Alice key from the dev seed
using "//Alice" as the derivation path on an sr25519 key.

Once the key is loaded, you should see that Alice has a large account balance.

To try out the multi-token module, go to https://polkadot.js.org/apps/#/extrinsics. You may need another dummy
account if you want to send PDock and PStable around. Unter the multiToken module in the extrinsics page, find
the transfer() function which will send tokens. Set `token_id` to true for PDock, false for PStable.
