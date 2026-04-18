TARGET ?= x86_64-unknown-linux-musl
VERSION := $(shell git describe --abbrev=0 --tags)
NAME := ledger-$(VERSION)-$(TARGET)
DOCKER_IMAGE ?= ledger-builder

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

# Open a shell in the builder container with local code mounted
docker-shell:
	docker run --rm -it -v $(PWD):/app $(DOCKER_IMAGE) /bin/sh

# Build the Docker image used for compilation
docker-image:
	docker build --target builder -t $(DOCKER_IMAGE) .

# Build a release binary using Docker (no local Rust/musl toolchain required)
docker-release:
	docker build --target builder -t $(DOCKER_IMAGE) .
	mkdir -p target/$(TARGET)/release
	docker run --rm $(DOCKER_IMAGE) cat /app/target/$(TARGET)/release/ledger > target/$(TARGET)/release/ledger
	chmod +x target/$(TARGET)/release/ledger
	rm -rf /tmp/$(NAME)
	mkdir /tmp/$(NAME)
	cp target/$(TARGET)/release/ledger /tmp/$(NAME)/
	cp README.md /tmp/$(NAME)/
	cp LICENSE /tmp/$(NAME)/
	tar zcf $(NAME).tar.gz -C /tmp $(NAME)
	rm -r /tmp/$(NAME)
	sha256sum $(NAME).tar.gz > $(NAME)-sha256sum.txt
	mkdir -p builds
	mv $(NAME).tar.gz $(NAME)-sha256sum.txt builds

# Also install rustup and musl
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
