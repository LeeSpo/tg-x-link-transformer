FROM rust:1-bookworm AS builder

ARG TARGETARCH

RUN apt-get update \
    && apt-get install -y --no-install-recommends musl-tools perl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN case "$TARGETARCH" in \
        amd64) rust_target="x86_64-unknown-linux-musl" ;; \
        arm64) rust_target="aarch64-unknown-linux-musl" ;; \
        *) echo "Unsupported TARGETARCH: $TARGETARCH" >&2; exit 1 ;; \
    esac \
    && rustup target add "$rust_target" \
    && cargo build --release --target "$rust_target" \
    && strip "/app/target/$rust_target/release/tg-x-link-transformer" \
    && mkdir -p /out \
    && cp "/app/target/$rust_target/release/tg-x-link-transformer" /out/tg-x-link-transformer

FROM alpine:3.21

RUN apk add --no-cache ca-certificates

WORKDIR /app
COPY --from=builder /out/tg-x-link-transformer /usr/local/bin/tg-x-link-transformer

ENV RUST_LOG=info

ENTRYPOINT ["tg-x-link-transformer"]
