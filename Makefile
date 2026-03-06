# Makefile
.PHONY: install build test clean

install:
	cargo install --path crates/sup

build:
	cargo build --release

test:
	cargo test --workspace

clean:
	cargo clean
