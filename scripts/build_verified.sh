#!/usr/bin/env bash
set -euo pipefail
export NO_COLOR=true
cd "$(dirname "$0")/.."
export NUSA_MODEL_PROFILE="${NUSA_MODEL_PROFILE:-detectai-topic-v1}"
if [[ "$NUSA_MODEL_PROFILE" == journal-pilot-v1 ]]; then
    python3 - "${NUSA_JOURNAL_MODEL_PATH:-training/runs/journal-three-pilot-2026-11-19/journal-pilot.json}" <<'PY'
import hashlib, sys
from pathlib import Path

expected = "9ae5b9533771c53c0bdeedd1aa705ff9367d011a28eceecafa66a2d0fdf88ed6"
if hashlib.sha256(Path(sys.argv[1]).read_bytes()).hexdigest() != expected:
    raise SystemExit("journal-pilot-v1 does not match the evaluated model file.")
PY
fi
cargo build --manifest-path worker/core/Cargo.toml --release --target wasm32-unknown-unknown --locked
# Must match Cargo.lock. Install with cargo install wasm-bindgen-cli --version 0.2.105 --locked if absent.
"${WASM_BINDGEN_BIN:-wasm-bindgen}" worker/core/target/wasm32-unknown-unknown/release/receipt_core.wasm --target bundler --out-dir worker/pkg
trunk build --release --locked --features verified-receipts
python3 scripts/validate_dist.py
printf 'verified-receipts-v1\n%s\n' "$NUSA_MODEL_PROFILE" > dist/_verified-build
npm ci --prefix worker
