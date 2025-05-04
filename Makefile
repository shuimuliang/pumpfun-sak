# Define the target triple for macOS M1
TARGET = aarch64-apple-darwin

# Define the output binary name
BINARY = sniping-server
WSProducer = enhanced_websocket_redis_producer
WSConsumer = enhanced_websocket_redis_consumer

.PHONY: all check clean

all: build

check: fmt test clippy

test:
	(command -v cargo-nextest && cargo nextest run --all-features --workspace) || cargo test --all-features --workspace

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets --tests -- -D warnings

clean:
	cargo clean

build:
	cargo build --release

copy-lib:
	cp target/release/libpumpfun_transaction_builder.dylib ~/cohuman/pump-fun/pumpfun-pylib/libpumpfun_transaction_builder.dylib

update-submodule:
	git submodule update --remote

build-sniping-server: update-submodule
	rustup target add $(TARGET)
	cargo build --release --target $(TARGET)
	cp target/$(TARGET)/release/$(BINARY) /tmp/$(BINARY)

build-websocket-hooks-server-linux: update-submodule
	rustup target add aarch64-unknown-linux-gnu
	cargo build --release --target aarch64-unknown-linux-gnu
	cp target/aarch64-unknown-linux-gnu/release/$(WSProducer) /tmp/$(WSProducer)
	cp target/aarch64-unknown-linux-gnu/release/$(WSConsumer) /tmp/$(WSConsumer)

bench:
	cargo bench --package helius-ws-hooks
