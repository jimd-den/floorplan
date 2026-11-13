CARGO ?= cargo

.PHONY: check build test clippy wasm

check: test clippy

test:
	$(CARGO) test -- --test-threads=1

clippy:
	$(CARGO) clippy --all-targets -- -D warnings

build:
	$(CARGO) build --release

# Builds the WASM adapter and verifies it against the native CLI.
wasm:
	$(CARGO) build
	RUSTFLAGS="-C panic=abort" $(CARGO) build --lib --target wasm32-unknown-unknown
	node tools/wasm_check.mjs
