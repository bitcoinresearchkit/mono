FROM rust:1.93-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates

RUN cargo build --release --locked -p bitviewd

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates procps && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash bitview

COPY --from=builder /app/target/release/bitviewd /usr/local/bin/bitviewd

USER bitview

RUN mkdir -p /home/bitview/.bitview

EXPOSE 3110

WORKDIR /home/bitview

ENTRYPOINT ["bitviewd"]
