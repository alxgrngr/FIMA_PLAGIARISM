#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Deliberate gate: never replace the live static site with an unprovisioned backend.
: "${CLOUDFLARE_ACCOUNT_ID:?Set Cloudflare account ID securely}"
: "${CLOUDFLARE_API_TOKEN:?Set Cloudflare API token securely}"
: "${NUSA_VERIFIED_CONFIG:?Path to reviewed private Worker config with actual D1 binding}"
: "${NUSA_FREE_CPU_VALIDATED:?Record deployed Free-tier CPU validation before production switch}"
test "$NUSA_FREE_CPU_VALIDATED" = "yes"
python3 - "$NUSA_VERIFIED_CONFIG" <<'PY'
import sys,tomllib
from pathlib import Path
p=Path(sys.argv[1]);d=tomllib.loads(p.read_text());db=d['d1_databases'][0]
assert db['binding']=='DB' and db['database_id']!='00000000-0000-0000-0000-000000000000','Configure the actual D1 binding first'
assert (p.parent / d['main']).resolve() == Path('worker/gateway.js').resolve(), 'Use the Durable Object gateway entry point'
assert any(b['name'] == 'RECEIPT_PROCESSOR' and b['class_name'] == 'ReceiptProcessor'
           and 'script_name' not in b for b in d['durable_objects']['bindings']), 'Bind the local ReceiptProcessor class'
assert any('ReceiptProcessor' in m.get('new_sqlite_classes', [])
           for m in d['migrations']), 'Include the SQLite Durable Object migration'
assert Path('dist/_verified-build').read_text().strip()=='verified-receipts-v1','Build verified artifacts first'
PY
npm exec --prefix worker -- wrangler deploy --config "$NUSA_VERIFIED_CONFIG"
