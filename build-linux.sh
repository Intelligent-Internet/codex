#!/bin/bash
# Build Linux binaries on Linux

set -euo pipefail

echo "Building Linux binaries..."

# Build for x86_64-unknown-linux-musl
cd codex-rs
cargo build --release --target x86_64-unknown-linux-musl

# Build for aarch64-unknown-linux-musl (ARM64)
rustup target add aarch64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl

echo "Linux binaries built successfully!"
echo "x86_64 binary: codex-rs/target/x86_64-unknown-linux-musl/release/codex"
echo "aarch64 binary: codex-rs/target/aarch64-unknown-linux-musl/release/codex"

# Copy to staging directory for npm package
mkdir -p /tmp/codex-staging/vendor/x86_64-unknown-linux-musl/codex
mkdir -p /tmp/codex-staging/vendor/aarch64-unknown-linux-musl/codex

cp target/x86_64-unknown-linux-musl/release/codex \
   /tmp/codex-staging/vendor/x86_64-unknown-linux-musl/codex/

cp target/aarch64-unknown-linux-musl/release/codex \
   /tmp/codex-staging/vendor/aarch64-unknown-linux-musl/codex/

echo "Binaries copied to staging directory"