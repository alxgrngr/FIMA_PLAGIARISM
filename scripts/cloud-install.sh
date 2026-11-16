#!/usr/bin/env bash
set -euo pipefail
export CARGO_HOME=/workspace/tooling/cargo
export RUSTUP_HOME=/workspace/tooling/rustup
export PATH="$CARGO_HOME/bin:$PATH"
export NO_COLOR=true
export XDG_CACHE_HOME=/workspace/tooling/cache
cd /workspace/FIMA_PLAGIARISM
if ! command -v rustup >/dev/null; then
  mkdir -p /workspace/tooling
  curl --fail --show-error --silent https://sh.rustup.rs -o /workspace/tooling/rustup-init.sh
  sh /workspace/tooling/rustup-init.sh -y --no-modify-path --profile minimal --default-toolchain 1.100.0
fi
rustup target add wasm32-unknown-unknown
rustup component add rustfmt clippy
cargo fetch --locked
if ! trunk --version 2>/dev/null | grep -q '0.21.14'; then
  cargo install trunk --version 0.21.14 --locked
fi
cargo test --locked
trunk build --release --locked
