# Cursory Repository Overview
	
Project structure is derived from node-template on substrate master. The main differences being:

- Actual chain execution is delegated to a pinned version of the `substrate` command.
- Custom runtime modules are isolated in the ./modules subdirectory. Each module is a cargo
  workspace member, and can be tested independently. The structure intends to make modules
  reusable by separating them from the runtime.

## Cargo workspace Structure

The cargo workspace contains several parts:

- ./
- ./runtime
- ./modules/erc20
- ./modules/multi-token
- ./modules/voting

## Chaingen

The root project, `substrate-warmup-chaingen`, is an executable that does nothing but generate
a chainspec. That chainspec is executed using a pinned version of substrate.

## Runtime

Like node-template, runtime is compiled to wasm. It also exports genesis types for use in
chaingen. Unlike in node-template, the native runtime is not used.

## Erc20 Module

The erc20 module is based on substrate-developer-hub/erc20-multi. Many improvements have been made:
tests, genesis configuration, at least one bug fix, a `burn()` method, etc. This module changes
`init()` to be super-user only.

## Multi-token

A stripped down version of erc20. In the multi-token module, token types are declared at compile
time as a generic `Discriminant` type. Unlike in the erc20 module, initial token allocations are
declared only in the chainspec. The only exposed method is the well tested `transfer()`.

## Voting Module

`./modules/voting` is imported from edgeware-voting. The module is included in the runtime, but
it's mostly untouched and unused for now.

# Other files

## ./ui-types.json

In the absence of a typed jsonrpc rust client (WIP). Chain interactions are manually tested using
the web ui at polkadot.js.org/apps. ui-types.json declares the custom types needed for submitting
extrinsics and querying state for this runtime.

## CI tests

Rust tests are run using github actions (./.github/workflows/test.yml) whenever a commit is pushed.
Test are run in the docker container `docker://parity/rust-builder:latest`.

## Dockerfile

A docker recipe is included for ease of use.

## Docker Compose

./docker-compose.yml declares a test network of multiple nodes.

# Msc

# Imports

Substrate crates are imported individually from the substrate repo. Each crate import references
our pinned substrate version:

```toml
[dependencies.substrate-consensus-babe-primitives]
git = "https://github.com/paritytech/substrate.git"
rev = "870b976bec729aaf26cc237df9fd764b8f7b9d7e"
```

We hope to expose all substrate modules from a single super-crate:

```toml
substrate = { path = "./substrate-proxy" }
```

getting that to work is non-trivial, so it's a WIP.

# Integration tests

High-level tests must be performed manually. We are implementing a typed jsonrpc client (WIP) which
will be used in multi node, chain level tests.
