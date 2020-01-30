all:
	@echo Nothing to do...

build:
	cargo build

debug:
	cargo build --verbose
	rustc -L ./target/deps/ -g -Z lto --opt-level 3 src/main.rs

dev:
	cargo build
	target/debug/ledger

fmt:
	cargo fmt

clippy:
	cargo clippy
