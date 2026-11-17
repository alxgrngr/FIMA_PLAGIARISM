#!/usr/bin/env bash
set -euo pipefail
export NO_COLOR=true
cd "$(dirname "$0")/.."
cargo build --manifest-path worker/core/Cargo.toml --release --target wasm32-unknown-unknown --locked
# Must match Cargo.lock. Install with cargo install wasm-bindgen-cli --version 0.2.105 --locked if absent.
"${WASM_BINDGEN_BIN:-wasm-bindgen}" worker/core/target/wasm32-unknown-unknown/release/receipt_core.wasm --target bundler --out-dir worker/pkg
trunk build --release --locked --features verified-receipts
python3 scripts/validate_dist.py
printf 'verified-receipts-v1\n' > dist/_verified-build
npm ci --prefix worker
