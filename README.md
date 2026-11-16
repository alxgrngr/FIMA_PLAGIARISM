# NUSA AI Detector

Rust + Leptos CSR / WebAssembly research prototype for Indonesian text. **Experimental detector, not a validated academic tool.** A real trained Indonesian TF-IDF/logistic model is available in opt-in research mode. Internal test accuracy is 94.31% on a narrow Wikipedia–Gemma dataset; this is not a general accuracy guarantee. See docs/MODEL_CARD.md.

## Available
Local paste input, UTF-8 TXT picker (400 KB limit), editable preview, explicit check action, word/character counts, NFC-normalized paragraph previews with original byte ranges in the core API, lexical diversity, clear/reset, responsive Indonesian UI, privacy and limitations notices. No document networking, telemetry, accounts or persistent document storage.

## Development
Rust 1.100.0, wasm32-unknown-unknown target, Trunk 0.21.14.

```
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.21.14 --locked
trunk serve
```

For this managed cloud environment, run `bash scripts/cloud-install.sh`; it also sets writable tool/cache paths and NO_COLOR=true.

## Checks and production
```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
trunk build --release
```

Output is `dist/`, for static Cloudflare Pages hosting. See docs/DEPLOYMENT.md. No public deployment has been authorized. Core logic is a native-testable library within the same package; the binary contains Leptos UI. CSS is local and system fonts avoid external asset requests.

## Model preparation and limitations
See docs/MODEL_CARD.md. The small research model is bundled and runs in Rust/WASM on CPU. Eligible paragraphs show uncalibrated scores and word-weighted flagged percentages after opt-in. No calibration, independent academic validation, report export, DOCX/PDF parser, OCR, language classifier, WebGPU, or offline guarantee is provided. Browser test coverage and measured checks are recorded in docs/PROGRESS.md. Paragraph boundaries currently follow nonempty lines, not layout-aware document paragraphs. Token count is not estimated without a selected tokenizer. Lexical diversity uses whitespace-separated, case-folded forms; punctuation is retained.

## Security and troubleshooting
File extension is only an initial filter; TXT content rejects NUL and replacement characters. This is conservative and rejects some legitimate Unicode text. Browser `File.text` decodes UTF-8. Limits bound input processing; no unsafe HTML insertion is used. See docs/SECURITY.md. Network restrictions may prevent toolchain or crate downloads; do not disable TLS verification. Model-loading failures must never be replaced with dummy scores.

## Zero-cost requirement
Use Cloudflare Pages Free and its included `*.pages.dev` address. No purchased domain, VPS, paid AI API, cloud database or rented GPU is authorized. Provider quotas still apply; never automatically upgrade to a paid plan.
