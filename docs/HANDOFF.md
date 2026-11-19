# Agent handoff — 2026-11-18

This is a curated project handoff, not a complete transcript. Read AGENTS.md first, then this file, PROGRESS.md and VERIFIED_RECEIPTS.md. Earlier progress entries describe historical states; use the latest entries for current status.

## Production update — 2026-11-19 WIB

Login and automatic receipts are live at https://fima-plagiarism.fiqihh413.workers.dev/. Earlier statements below about the main site being static or the roster being local describe historical states. The live version was built from the receipt-cpu-optimization branch with the Durable Object gateway and a separate production D1 roster. Browser and API smoke tests with the designated test account and synthetic 10 MiB DOCX passed. See the latest PROGRESS.md entry. Future production deploys should use the ignored private/production.toml and scripts/deploy_verified.sh. The root Wrangler config remains the historical static deployment and must not be used to update the active verified Worker.

## Active model update — 2026-11-19 WIB

The live site now uses `journal-pilot-v1` in both browser and server for new reports. Build the verified app with `NUSA_MODEL_PROFILE=journal-pilot-v1` and the private trained artifact described in VERIFIED_RECEIPTS.md. The staged and production parity/receipt checks passed. Earlier descriptions below refer to the former model or static prototype.

## User requirements

- Indonesian-language AI detector for Perilaku Organisasi practicum reports, Teknik Industri, Universitas Muhammadiyah Malang. Simple industrial-blue design, Rust/Leptos CSR and WASM.
- Everything must stay FREE, including hosting/domain. Never enable paid services to resolve limits.
- DOCX up to 10 MiB and TXT; extracted text limit 100,000 characters. Initial preview/analysis stays in browser.
- NIM-only access from uploaded roster: sheet 6 students (21), sheet 7 test account (1). User explicitly accepted NIM impersonation risk. Do not include other sheets without instruction.
- User consented to temporary server processing for authoritative receipts and D1 storage of identity/receipt metadata, NOT document contents.
- Receipt: identity, group, filename, result, server time in WIB, timestamp watermark, QR verification, instruction to screenshot for aslab. Screenshots/DOM cannot be made immutable; server signature and verification record detect altered claims. Public verification masks identity.

## Repository and environments

- Repository: https://github.com/alxgrngr/FIMA_PLAGIARISM
- Checkout: /workspace/FIMA_PLAGIARISM, local branch work; previously pushed with git push origin HEAD:main.
- Last implementation/diagnostics commit before this handoff: 4965878, confirmed pushed.
- Main URL: https://fima-plagiarism.fiqihh413.workers.dev/
- Separate test Worker: https://nusa-receipts-staging.fiqihh413.workers.dev/
- Main configuration remains static. verified-receipts is OFF by default; remote NIM restriction/receipts are NOT activated on main.
- Staging has a real D1 binding and signing secret, with SYNTHETIC participants only. Real roster has only been imported into LOCAL D1.

## Implemented and tested

- Research model: TF-IDF logistic regression trained on 2,240 DETECTAI Indonesian Wikipedia/Gemma samples, topic-separated splits. Internal test accuracy 94.31%; not validation on practicum reports. Poor calibration: never label scores as authorship probabilities. See MODEL_CARD.md.
- Rust inference, DOCX extraction, local previews and research scoring.
- Auth frontend in src/auth.rs; server routing/auth/signing in worker/src.js; Rust server core in worker/core; D1 migration in worker/migrations.
- Session cookies, allowlist, origin checks, issuance/login limits, original-file hashing, server recomputation, HMAC-signed records, owner receipt and masked public verification.
- Local Rust/Clippy/build, API/browser, synthetic roster tests passed as recorded in PROGRESS.md. Remote staging login/receipt tests passed, including approximately 99k characters and an exactly 10 MiB DOCX. Do not equate functional success with Free-tier feasibility.

## Staging update — 2026-11-19 WIB

The receipt-cpu-optimization branch now includes a SQLite Durable Object gateway. It supersedes the direct Worker processing below and keeps issuance automatic at the website. Synthetic staging passed three simultaneous 10 MiB uploads. The gateway reported 44 requests, zero errors and a maximum group p99 of 2.088 ms. The processor analytics returned no rows at collection time. See the latest PROGRESS.md and benchmark before changing production. The evidence and proposed staged processing below describe the earlier direct Worker design.

## Current blocker — do not bypass

Cloudflare analytics on 2026-11-17: 17 staging requests, zero errors; CPU median 4.397 ms and p99 72.104 ms. Two HTTP 201 receipt-issuance requests consumed 61.066 and 72.104 ms. These exceed the intended Workers Free 10 ms CPU budget. HTTP 200 groups also reached 37.970 ms, without route attribution. Units verified from GraphQL schema (microseconds). Small adaptive sample, not a load test.

- Evidence: benchmarks/cloudflare-staging-2026-11-17.json and cloudflare-staging-cpu-groups-2026-11-17.json.
- Repeatable read-only collection: python scripts/cloudflare_cpu.py --since 2026-11-17T00:00:00Z.
- performance.now()/Server-Timing returned zero for synchronous remote inference; do not use it as billed CPU evidence. Startup time is also not request CPU.
- Do NOT set NUSA_FREE_CPU_VALIDATED=yes or switch main to verified mode based on these tests.

Latest proposed solution in chat: split authoritative server processing into bounded requests, with more requests and potentially longer issuance time. This is ONLY A PROPOSAL, not implemented or proven. A correct design must bound parsing, decompression, inference and hashing, bind all chunks/order/completeness to the same original document and user, and resist client manipulation/replay. Signing a browser-supplied percentage or trusting client-extracted text does not meet the verification requirement. Do not persist document contents without new authorization. Optimizing QR alone cannot fix the measured issuance cost. Do not silently reduce the accepted upload limit or upgrade billing.

## Private state and secrets

- private/roster.sql: ignored real roster seed. private/staging.toml, staging-state.json and staging-users.json: ignored local staging config/synthetic fixtures. These may not transfer to a fresh workspace.
- Uploaded workbook is outside the repository under /workspace/attachments; original upload ID file_0000000057888211b8e7562cc0039583. If unavailable in a new session, request secure reattachment, never paste roster into public Git.
- CLOUDFLARE_ACCOUNT_ID and CLOUDFLARE_API_TOKEN were supplied via secure environment settings. Do not print or commit values. Fresh agents/environments might not inherit them.
- RECEIPT_SIGNING_KEY is a Cloudflare Worker secret; do not rotate it casually, since existing receipt verification depends on it.
- No complete conversation transcript or secret store is committed. This handoff and linked docs are the portable project memory.

## Useful commands and cautions

- Build verified assets: bash scripts/build_verified.sh. Requires Rust/Trunk/wasm-bindgen and npm dependencies described in project setup. Default Trunk build replaces dist with the static frontend.
- worker/wrangler.toml is LOCAL with a placeholder DB. Do not use it for production deployment.
- Production deployment wrapper scripts/deploy_verified.sh intentionally requires a reviewed config and CPU validation.
- Staging API runner: NODE_USE_ENV_PROXY=1 node tests/receipts-staging.cjs (uses private synthetic users).
- Tools were installed under /workspace/tooling: CARGO_HOME=/workspace/tooling/cargo, RUSTUP_HOME=/workspace/tooling/rustup; prepend CARGO_HOME/bin to PATH. wasm-bindgen used version 0.2.105.
- Keep private data out of test output, commits and screenshots. Inspect current git state before edits; do not reapply completed work or publish an environment draft automatically.

## Next task

User is handing this project to another agent. Explain current state honestly, then design/implement and measure a Free-compatible authoritative receipt pipeline if asked to continue. Preserve working static production until the replacement passes functional, integrity and deployed CPU tests. The full website goal remains unfinished because verified receipts/login have not been activated on main.
