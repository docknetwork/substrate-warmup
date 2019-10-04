# Substrate Warmup

A Parity Substrate SRML baseline module for DockChain. The primary purpose is to work toward a core
chain and module config, which includes a multi token module and a voting module.

# Quick setup using Docker

```bash
docker build -t full-node .
docker run -p 9944:9944 full-node --alice
```

See the Dockerfile for more details.

# Quick p2p swarm simulation using docker-compose

```bash
docker-compose up --scale standard=5
#                                  ^ number of default nodes to simulate
```

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
(cd substrate; git checkout 870b976bec729aaf26cc237df9fd764b8f7b9d7e) # our current pinned version
cargo install --path substrate --force
```

## Run

You can run the dev testnet chain with:

```bash
# make a temporary directory in which to store chain data
mkdir -p tmp

# create a chainspec
cargo run --release -- ved > tmp/chainspec.json
#         ^^^^^^^^^    ^^^ ^^^^^^^^^^^^^^^^^^^^ Dump the chainspec into a file which we'll use in
#             |         |                       the next step.
#             |         |
#             | Specify the dev chain. Run `cargo run --release -- help`
#             | for a full list of options.
#             |
# The runtime is executed purley in Wasm. The naitive runtime is disabled for this chain.
# Wasmi sometimes can't keep up with block production unless compiled with optimizations.
# In addition to the being slow, the runtime much larger when compiled without --release.

# run created chainspec using substrate
substrate --chain ./tmp/chainspec.json --alice --base-path ./tmp
#         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ^^^^^^^ ^^^^^^^^^^^^^^^^^ Store chain data in a
#             |                           |                      temporary directory.
#             |                           |
#             | Use the publicly known keypair, 'Alice', to produce blocks.
#             |
# Run the chain specification we specified in the previous command.
```

# Using the polkadot js UI

Once the dev chain is running, natively; within docker; or through docker-compose, you can interact
with it via browser.

Go to https://polkadot.js.org/apps/#/settings and set "remote endpoint" to your locally running node 127.0.0.1.

Copy the type definitions in "./ui-types.json" to https://polkadot.js.org/apps/#/settings/developer and save.

To interact with the dev chain, you'll need to load the private key for the superuser, Alice, into the browser ui.
Go to https://polkadot.js.org/apps/#/accounts and add an account with the publicly known seed
`bottom drive obey lake curtain smoke basket hold race lonely fit walk`. Derive the Alice key from the dev seed
using "//Alice" as the derivation path on an sr25519 key.

Once the key is loaded, you should see that Alice has a large account balance.

To try out the erc20 module, go to https://polkadot.js.org/apps/#/extrinsics. You may need another dummy
account if you want to send PSTABLE1 and PSTABLE2 around. Under the erc20 module in the extrinsics page, find
the transfer() function which will send tokens. Set `token_id` to 0 for PSTABLE1, 1 for PSTABLE2.

# Note about Licensing

Dock's Substrate development is currently licensed under GPLv3 as inherited from paritytech/substrate. This license will change to Apache 2.0 in sync with Parity Technologies updating the entire Substrate toolset to Apache 2.0 as previously announced.
