# You can run this docker image with:
#
#   docker build -t dev-full-node .
#   docker run -p 30333:30333 -p 9933:9933 -p 9944:9944 dev-full-node --alice
#                 ^^^^^^^^^^^    ^^^^^^^^^    ^^^^^^^^^ ^^^^^^^^^^^^^ ^^^^^^^
#                    |              |             |          |           |
#                 p2p port       rpc port     ws rpc port    |  create blocks using publicly
#                                                            |  knows keypair named Alice
#                                                            |
#                                          tag for the docker image we just created
#
#
# Test whether the container successfully exposed its rpc server:
#
#   curl                                                            \
#     -H "Content-Type: application/json"                           \
#     -d '{"id":1, "jsonrpc":"2.0", "method": "state_getMetadata"}' \
#     http://localhost:9933
#
# 
# To use a different chain specification set "chain_generator_args" when running docker build.
#
#   docker build --build-arg chain_generator_args=help -t dev-full-node .
#                                                 ^^^^ Set this to "help" to see available options
#
#
# To create and run a chain with custom a verifier, root-key, and treasury
#
#   subkey generate # subkey is often used for generating keypairs
#   mypk=0x662b1ce11aaf35fad3e7b188f9ba28eb4f1cab3f0b991e587e294d6a6c58c332
#   docker build --build-arg chain_generator_args="custom $mypk $mypk $mypk $mypk" -t dev-full-node .
#
#
# For more customization, override the docker entrypoint: (example shown, disabling websocket rpc)
#
#   docker run                               \
#     -p 30333:30333                         \
#     -p 9933:9933                           \
#     --entrypoint /usr/local/bin/substrate  \ <- This is where the substrate executable is stored.
#     dev-full-node                          \
#     --rpc-cors all                         \
#     --rpc-external                         \
#     --chain /chainspec.json                # <- Don't forget the chainspec path. Otherwise the
#                                            #    container will run the wrong chain.

# -------------- Build substrate ---------------- #

from parity/rust-builder:d53c78d1-20191118 as substrate-builder

workdir /
run git clone https://github.com/paritytech/substrate.git
workdir substrate

run git checkout aa937d9b4e5767f224cf9d5dfbd9a537e97efcfc; cargo fetch
run cargo build --release --bin substrate --offline

# -------------- Build chainspec ---------------- #

from parity/rust-builder:d53c78d1-20191118 as chainspec-builder

copy . project
workdir project

run cargo fetch
run cargo build --release --offline

# set chain_generator_args to "help" to see available options
arg chain_generator_args=ved

# The user may run a command that doesn't generate a chain.
# If they do, we halt the build process and allow them to read the result.
run if [ "x${chain_generator_args}" = "xhelp" ] \
	|| [ "x${chain_generator_args}" = "x--help" ] \
	|| [ "x${chain_generator_args}" = "x-h" ] \
	|| [ "x${chain_generator_args}" = "xversion" ] \
	|| [ "x${chain_generator_args}" = "x--version" ] \
	|| [ "x${chain_generator_args}" = "x-V" ] \
	; then : \
    ;     cargo run --release --offline -- help \
	;     exit 1 \
	; fi

run cargo run --release --offline -- $chain_generator_args > /chainspec.json

# -------------------- Run ---------------------- #

from debian:stretch-slim

copy --from=chainspec-builder /chainspec.json /chainspec.json
copy --from=substrate-builder /substrate/target/release/substrate /substrate

entrypoint [          \
	"/substrate",     \
	"--rpc-cors",     \
	"all",            \
	"--rpc-external", \
	"--ws-external",  \
	"--chain",        \
	"/chainspec.json" \
]
