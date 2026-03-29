FROM rust:alpine AS builder

RUN apk add --no-cache \
    musl-dev \
    libsodium-dev \
    pkgconfig \
    git \
    perl \
    make

WORKDIR /app

# Cache dependencies by building a dummy binary first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl; \
    rm -rf src

COPY src ./src
# Touch to ensure the source is rebuilt even if Cargo.toml cache is reused
RUN touch src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/ledger /ledger
ENTRYPOINT ["/ledger"]
