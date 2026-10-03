#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TOOLCHAIN_FILE="$SCRIPT_DIR/../rust-toolchain.toml"

rustup update

# Update rust-toolchain.toml with current stable version
RUST_VERSION=$(rustup run stable rustc --version | awk '{print $2}')
echo "[toolchain]" > "$TOOLCHAIN_FILE"
echo "channel = \"$RUST_VERSION\"" >> "$TOOLCHAIN_FILE"
echo "Updated rust-toolchain.toml to $RUST_VERSION"

cargo clean
cargo upgrade --incompatible
cargo update
cargo check --workspace
