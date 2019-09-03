all:
	@echo Nothing to do...

build:
	OPENSSL_INCLUDE_DIR=/usr/include/openssl-1.0 OPENSSL_LIB_DIR=/usr/lib/openssl-1.0 cargo build

debug:
	cargo build --verbose
	rustc -L ./target/deps/ -g -Z lto --opt-level 3 src/main.rs

dev:
	cargo build
	target/debug/ledger
