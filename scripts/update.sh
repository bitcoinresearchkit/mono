#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TOOLCHAIN_FILE="$SCRIPT_DIR/../rust-toolchain.toml"

rustup update

RUST_VERSION=$(rustup run stable rustc --version | awk '{print $2}')
echo "[toolchain]" > "$TOOLCHAIN_FILE"
echo "channel = \"$RUST_VERSION\"" >> "$TOOLCHAIN_FILE"
perl -pi -e "s/^FROM rust:[0-9.]+-/FROM rust:$RUST_VERSION-/" "$SCRIPT_DIR/../docker/Dockerfile"
echo "Updated rust-toolchain.toml and docker/Dockerfile to $RUST_VERSION"

cargo clean
cargo upgrade --incompatible
cargo update
cargo check --workspace
