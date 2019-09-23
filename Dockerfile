from parity/rust-builder:latest as builder

copy . /project

run cd /project && cargo build --release

# Strip all the build dependencies away and start fresh.
from ubuntu:latest

copy --from=builder /project/target/release/node-template /usr/local/bin/node-template

run apt-get update \
	&& apt-get install libssl-dev -y

cmd /usr/local/bin/node-template --chain=ved
