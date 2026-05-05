FROM rust:1-alpine AS builder

RUN apk add --no-cache build-base

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

ENV RUSTFLAGS="-C target-feature=+crt-static"

RUN cargo build --release \
    && strip /app/target/release/tg-x-link-transformer

FROM alpine:3.21

RUN apk add --no-cache ca-certificates

WORKDIR /app
COPY --from=builder /app/target/release/tg-x-link-transformer /usr/local/bin/tg-x-link-transformer

ENV RUST_LOG=info

ENTRYPOINT ["tg-x-link-transformer"]
