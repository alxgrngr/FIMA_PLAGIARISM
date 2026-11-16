# PROGRESS

Initial audit: empty unborn Git branch work; no source, documentation or instructions to preserve. GitHub read succeeds. Rust absent initially, installed under /workspace/tooling. Source implementation: UI and native text processing IMPLEMENTED; TXT picker IMPLEMENTED. Unit/build results recorded below. Model/dataset research BLOCKED by missing verified data and restricted research destinations. Burn POC, DOCX, PDF, report, cache, offline, GPU, classifier and training NOT STARTED. Public deployment NOT STARTED. No MVP readiness claim.

## Verification in managed Linux environment
- TESTED: `cargo test --locked`: 5 library tests passed, no failures. Binary and doc-test targets contain zero tests.
- TESTED: `cargo fmt --check` and `cargo clippy --locked --all-targets -- -D warnings` pass.
- TESTED: `cargo build --locked --release --target wasm32-unknown-unknown` succeeds.
- Dependency warning: proc-macro-error2 2.0.1 has future Rust incompatibilities; current pinned toolchain compiles. Trunk 0.21.14's upstream locked dependency crossbeam-channel 0.5.14 is yanked; installation warning requires tooling dependency review before release.
- Saved environment install_script/start_skill draft; not published.
- TESTED: reproducible setup script, Trunk release build, and Chromium smoke test with production CSP headers all pass.

- TESTED: Chromium paste/counts, paragraph display, TXT read, invalid/binary TXT, oversized file, unsupported PDF rejection, clear/empty error and 390px mobile overflow checks. No page errors.
- Final WASM: 289,673 bytes; JS: 31,067 bytes. All output files below 25 MiB. Runtime latency, peak memory, Firefox/Edge/Safari, GPU and inference NOT TESTED.
- Overall delivery is partial, not the requested complete detector MVP. Remaining independent work includes DOCX/PDF parsing, reports, statistics expansion, accessibility coverage and model research/training tooling.

## Follow-up: repository push and ML tooling
Initial prototype pushed to main at c8107b8. Deterministic TF-IDF/logistic baseline pipeline IMPLEMENTED; five pipeline software tests TESTED (synthetic fixtures only). Real-data training remains BLOCKED pending approved, accessible dataset. M4 candidate audited; no explicit license found in inspected repository. Manual Cloudflare deployment workflow IMPLEMENTED, NOT TESTED externally. No Cloudflare credential bindings found in runtime. Network research domain additions saved as draft, not published.

## Blank deployment investigation
User screenshot shows NUSA title with blank body. Live fetch is blocked by environment proxy, so root cause is not confirmed. Source root index.html has an empty body and requires Trunk compilation. Added explicit Workers dist-only asset configuration, portable Cloudflare build script and fail-closed compiled asset validator. Build and validator pass locally. No external redeploy performed.

## Rust ML runtime
Added bounded JSON model loading, schema/coefficient/index validation, TF-IDF and CPU logistic inference matching the Python baseline formula. Unknown-vocabulary input abstains instead of returning bias-only scores. Research models remain disabled in the public UI. Tests use explicit mathematical fixtures, not claimed trained detector weights. Dataset access now works; official SemEval upstream is subject to a DMCA takedown, so its unofficial mirror was not used. No real training/evaluation metrics are claimed.
Validation: cargo test (10 passed), cargo fmt --check, cargo clippy --locked --all-targets -- -D warnings and Trunk release WASM build pass. Python-export/native Rust inference parity passes on three synthetic fixtures within 1e-12. This does not validate real-data quality, browser inference performance or GPU support. No weights are distributed. Existing proc-macro-error2 future-compatibility warning remains.

## Current result: real model integration (supersedes earlier unavailable status)
- TESTED: actual topic-grouped training on 2,240 DETECTAI records, 1,348 train / 453 validation / 439 test. 150 disjoint topic groups; no exact normalized duplicates.
- TESTED: held-out test TP=200, FP=6, TN=214, FN=19; accuracy 94.31%, F1 94.12%, FPR 2.73%. This is an internal Wikipedia/Gemma benchmark, not general academic validity. Poor ECE=0.31630 means scores must not be called calibrated probabilities.
- IMPLEMENTED/TESTED: opt-in research UI, actual per-paragraph Rust/WASM scores, eligible-word weighted flagged fraction, short/long paragraph exclusions, explicit model limitations and threshold.
- TESTED: 10 Rust unit tests, 1 bundled-weight native parity integration test, 5 Python pipeline tests, fmt/clippy, release build and artifact size validation.
- TESTED: Chromium production CSP smoke; real model scores match Python on three held-out corpus records and two committed handwritten numerical fixtures. WebGPU disabled; no external requests. Mixed-paragraph aggregation and exclusion checks pass. The fixtures do not constitute quality evaluation.
- Observed browser click-to-result times 47–125 ms for small cases include test automation and rendering; not an inference-only benchmark. Model JSON 587,114 bytes; production WASM 932,284 bytes. Every deployment file <25 MiB.
- NOT TESTED: Firefox/Edge/Safari, GPU, peak memory, unseen generators, independent thesis data, human-edited AI, domain-shift robustness. DOCX/PDF and report export remain unimplemented.
- Public deployment of this new model not verified. Commit/push delivers code; Cloudflare must finish deployment before the research UI is visible.
- TESTED: second independent execution of the same pinned preparation/training commands reproduced byte-identical model weights (`cmp` exit 0) and identical test metrics.
