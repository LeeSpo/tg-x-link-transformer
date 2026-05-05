FROM rust:1-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends musl-tools perl \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add x86_64-unknown-linux-musl

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --target x86_64-unknown-linux-musl \
    && strip /app/target/x86_64-unknown-linux-musl/release/tg-x-link-transformer

FROM alpine:3.21

RUN apk add --no-cache ca-certificates

WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/tg-x-link-transformer /usr/local/bin/tg-x-link-transformer

ENV RUST_LOG=info

ENTRYPOINT ["tg-x-link-transformer"]
