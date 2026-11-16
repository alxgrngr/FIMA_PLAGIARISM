#!/usr/bin/env bash
set -euo pipefail
export NO_COLOR=true
# Cloudflare build environments need Rust and Trunk; honor existing installations.
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
if ! command -v rustup >/dev/null; then
  installer=$(mktemp)
  curl --proto '=https' --tlsv1.2 --fail --show-error --silent https://sh.rustup.rs -o "$installer"
  sh "$installer" -y --profile minimal --no-modify-path --default-toolchain 1.100.0
  rm "$installer"
fi
rustup target add wasm32-unknown-unknown
if ! command -v trunk >/dev/null || ! trunk --version | grep -q '0.21.14'; then
  cargo install trunk --version 0.21.14 --locked
fi
trunk build --release --locked
python3 scripts/validate_dist.py
