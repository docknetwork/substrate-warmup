# based on https://github.com/paritytech/scripts/tree/master/dockerfiles/rust-builder

# DockerHub's official https://hub.docker.com/_/rust
FROM rust:slim-stretch

WORKDIR /builds

# install tools and dependencies
COPY clang8.key /etc/apt/trusted.gpg.d/debian-archive-jessie-automatic.gpg

# config for wasm32-unknown-unknown
COPY config /root/.cargo/config

# gcc and clang are necessary to be able to compile crates that have build scripts
# Most notably, clang 8 has WASM support enabled by default.
RUN set -eux; \
	echo "deb http://apt.llvm.org/stretch/ llvm-toolchain-stretch-8 main" \
		> /etc/apt/sources.list.d/llvm.list; \
	echo "deb-src http://apt.llvm.org/stretch/ llvm-toolchain-stretch-8 main" \
		>> /etc/apt/sources.list.d/llvm.list; \
	DEBIAN_FRONTEND=noninteractive apt-get -y update; \
	DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
		g++ libssl-dev gcc clang-8 cmake make \
		git pkg-config curl time rhash; \
# set a link to clang-8
	ln -s /usr/bin/clang-8 /usr/bin/clang; \
# install Rust nightly, default is stable
	rustup install nightly; \
# install cargo tools
	cargo install \
		cargo-audit cargo-web wasm-pack cargo-fmt cargo-clippy; \
# install wasm toolchain
	rustup target add wasm32-unknown-unknown; \
	rustup target add wasm32-unknown-unknown --toolchain nightly; \
	cargo +nightly install --git https://github.com/alexcrichton/wasm-gc; \
# versions
	rustup show; \
	cargo --version; \
# apt clean up
	apt-get autoremove -y; \
	apt-get clean; \
	rm -rf /var/lib/apt/lists/*;

ENV CC=gcc \
	CXX=g++ \
	RUST_BACKTRACE=1
