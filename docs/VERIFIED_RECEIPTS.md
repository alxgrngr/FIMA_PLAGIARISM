# Login and verified receipts — activation pending

## Implemented and tested locally
One NIM login page, server allowlist, opaque 8-hour HttpOnly/Secure/SameSite=Strict sessions, per-IP login limit (20/minute), per-NIM issuance limit (5/hour), explicit temporary-upload consent, server-side Rust DOCX/TXT parsing and model inference, D1-backed receipt records, server timestamp (WIB), SHA-256 of original uploaded file, HMAC-SHA256 signature, QR verification, and repeated timestamp watermark. Test-role accounts cannot manage users and always issue TEST receipts. Preview edits invalidate the original file selection; upload the saved original file again for a receipt.

NIM-only authentication is explicitly accepted by the owner. It does not prove who holds that NIM. Sessions are access controls, not university identity proof. Source code/WASM assets remain public; modifying the local UI cannot authorize server endpoints or issue signed records.

The original workbook has 7 sheets. Only sheet 6 (KELAS I: 21 participants) and sheet 7 (one test account) are selected per the owner's instruction. Sheets 1–5 are excluded. Group cells merged across rows are resolved. Import fails on duplicate NIM, invalid IDs or missing required fields. Names/NIMs are never printed by the import script. The private SQL is in ignored `private/roster.sql`, not Git or dist. Do not upload it as a website asset or CI artifact.

## Privacy and integrity boundary
Uploaded document bytes and extracted text exist only in transient request/WASM memory. There is no document storage API or document logging. This is not a forensic guarantee of instantaneous memory wiping. D1 stores participant records, hashed sessions, rate counters, and signed receipt payloads (identity, group, filename, file hash, timestamp, model/version, word counts, numeric segment results). No document text is stored. Metadata is retained for verification until deliberately removed by the operator; agree a semester retention policy before wider use. Scheduled cleanup removes expired sessions/rate counters hourly.

Owner receipt pages require the same student's session. Public verification requires an unguessable 256-bit ID, masks the NIM/name, and retains filename, group and outcome for checking. Anyone who receives the QR has access to that limited metadata. The server verifies HMAC before displaying a stored receipt. Key rotation must preserve the old verification key or old receipts become invalid; this v1 supports a single key, so do not rotate casually. Observability logging is disabled in the Worker config. Platform request metadata may still exist; document contents are never intentionally logged.

Screenshots and the DOM can always be altered. Watermarks are only visual aids. Authenticity comes from checking the QR against the live record at the correct official deployment hostname. Aslab must compare the original uploaded file's SHA-256 too; filename alone does not establish which document was checked. A genuine receipt does not prove the classifier is correct, or that the uploader owned the NIM.

## Local setup (tested)
1. Build Rust dependencies/target as in README. Install wasm-bindgen-cli 0.2.105 and Trunk 0.21.14, or set WASM_BINDGEN_BIN to the existing verified Trunk tool cache executable.
2. `bash scripts/build_verified.sh` builds a separate Rust WASM server core, then the Leptos frontend with `verified-receipts`, validates assets and installs locked worker dependencies.
3. Generate `worker/.dev.vars` with a random 32-byte-or-longer hex `RECEIPT_SIGNING_KEY`. Never commit it; do not reuse the local key in production.
4. `npm exec --prefix worker -- wrangler d1 migrations apply DB --config worker/wrangler.toml --local`
5. For synthetic tests only: `npm exec --prefix worker -- wrangler d1 execute DB --config worker/wrangler.toml --local --file=tests/receipt-seed.sql`
6. `npm exec --prefix worker -- wrangler dev --config worker/wrangler.toml --local --port 8787`
7. `node tests/receipts-api.cjs` and `node tests/receipts-browser.cjs`. Tests use synthetic users only. Rate counters can affect repeated tests; use an isolated local DB/reset only synthetic test state.

To generate a real roster import: `python3 scripts/import_roster.py /path/to/workbook.xlsx`. Install pinned importer dependencies with `python3 -m pip install -r scripts/requirements-roster.txt`. No spreadsheet contents are evaluated as code.

## Free-tier feasibility — NOT confirmed
Initial local inference wall timing was about 45ms before caching model initialization; later sample was 16ms with initialization moved to isolate startup. These are local workerd wall-clock observations, not Cloudflare billed CPU. D1, hashing, document parsing, HMAC and QR generation add work; large files/long reports can exceed a 10ms CPU budget. WASM is not exempt from Worker CPU limits. Local successful execution does not establish Free-tier deployment readiness.

Static assets bypass Worker execution except /api/*, /receipt/* and /verify/*. Login/issuance/verification consume Worker quota and D1 operations. No paid plan, paid AI API, rented GPU or paid domain is authorized. If measured deployed Free-tier CPU cannot support this design, do not auto-upgrade or silently issue browser-supplied scores. Stop and propose a clearly scoped alternative. Production activation remains blocked pending actual account provisioning and CPU measurements.

## Remote provisioning (NOT executed)
No Cloudflare token/account binding was present. A secure environment draft now requests CLOUDFLARE_API_TOKEN (scoped to the intended account with D1 edit and Workers Scripts edit capabilities) and CLOUDFLARE_ACCOUNT_ID. Token values must be supplied through secure settings, never chat/Git. api.cloudflare.com was added to the draft's allowed secret destinations. Saving that draft does not provision D1 or publish the website.

After authorized credentials are usable:
- Create a new D1 database via the account's Free plan, apply worker/migrations/0001.sql, and import private/roster.sql directly to D1 (never a public workflow artifact).
- Prepare an ignored private Worker config with the real D1 ID and correct absolute or relative paths to src.js/dist/migrations and the intended production Worker name. The checked-in config is named nusa-receipts-local to avoid accidentally replacing the live Worker. Do not modify the current live root wrangler.toml until tested.
- Set a newly generated production RECEIPT_SIGNING_KEY via Wrangler secret input or dashboard, not as a plaintext vars entry.
- Use a separate staging Worker on Free to test CPU usage for representative and boundary-size files, login, verification and QR generation; use synthetic participants there.
- Only after passing, set NUSA_VERIFIED_CONFIG to the reviewed production config and NUSA_FREE_CPU_VALIDATED=yes, then use scripts/deploy_verified.sh. The deployment script rejects the placeholder D1 ID and missing auth-build marker.

The existing root wrangler.toml and static build remain the default, so pushing the implementation does not silently replace the live website with a backend lacking its database. **The current public site does not gain NIM restrictions until this controlled activation is completed.**

### Provisioning update — 2026-11-17
Cloudflare API access now works. Staging database/Worker `nusa-receipts-staging` exists with schema, synthetic accounts and a Cloudflare-managed signing secret. Production D1/real roster activation has NOT occurred. Staging configuration and synthetic IDs are in ignored private files. Main Worker is untouched. To continue, apply/publish the saved environment network additions for `nusa-receipts-staging.fiqihh413.workers.dev` and `fima-plagiarism.fiqihh413.workers.dev`; then run the remote staging tests and obtain actual CPU-limit evidence before any production switch. The published staging address alone does not establish that issuance works within Free CPU limits.
