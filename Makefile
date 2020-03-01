TARGET ?= x86_64-unknown-linux-musl
VERSION := $(shell git describe --abbrev=0 --tags)
NAME := ledger-$(VERSION)-$(TARGET)

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

release:
	rustup target add $(TARGET)
	mkdir -p builds
	cargo build --release --target $(TARGET)
	rm -rf /tmp/$(NAME)
	mkdir /tmp/$(NAME)
	cp target/$(TARGET)/release/ledger /tmp/$(NAME)/
	cp README.md /tmp/$(NAME)/
	cp LICENSE /tmp/$(NAME)/
	tar zcf $(NAME).tar.gz -C /tmp $(NAME)
	rm -r /tmp/$(NAME)
	sha256sum $(NAME).tar.gz > $(NAME)-sha256sum.txt
	mv $(NAME).tar.gz $(NAME)-sha256sum.txt builds
