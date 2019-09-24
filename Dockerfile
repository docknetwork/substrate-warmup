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
# To build an image that defaults to a different chain, use the default_chain arg:
#
#   docker build --build-arg default_chain=ent -t dev-full-node .
#                                          ^^^
#                                           |
#                       see the ./chainspecs directory for available options
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
#     --chain /usr/share/chainspecs/ved.json # <- Don't forget the chainspec path. Otherwise the
#                                                 container will run the wrong chain.

from parity/substrate:2.0.0-870b976be

# see the ./chainspecs directory for available options
arg default_chain=ved

copy ./chainspecs/*.json /usr/share/chainspecs/
copy ./chainspecs/${default_chain}.json /usr/share/chainspecs/default.json

entrypoint [                             \
    "/usr/local/bin/substrate",          \
	"--rpc-cors",                        \
	"all",                               \
	"--rpc-external",                    \
	"--ws-external",                     \
	"--chain",                           \
	"/usr/share/chainspecs/default.json" \
]
