# detectai-topic-v1 — research-only Indonesian baseline

A real trained TF-IDF + logistic regression model is included in `models/detectai-topic-v1.json` and runs in Rust/WASM on CPU. It is opt-in, not a validated general-purpose authorship detector. It must not be used as evidence of academic misconduct.

## Provenance and rights
Dataset: https://huggingface.co/datasets/shouwiku/detectai-indonesian at revision `7ba07fffc32ec81b0d6e660a1ebfea9ec9400fc0`. Publisher declares MIT in the card. Citation: Kusuma, M. J. (2026), *Sistem Deteksi Teks Generatif AI Menggunakan Metode Hybrid TF-IDF Dan Stylometry Berbasis Logistic Regression*, Universitas Bandar Lampung, DETECTAI Hugging Face repository. Human-labelled samples are Wikipedia Indonesian excerpts; AI-labelled samples are attributed by the publisher to Gemma 4 variants. Labels and generation logs have not been independently authenticated. Despite the dataset title, Wikipedia is not a held-out collection of student theses. Original Wikipedia attribution/share-alike obligations are not replaced by a publisher MIT label. Corpus text is not redistributed in this repository. Preserve source metadata when conducting further research.

## Training
2,240 records; 1,120 per class; 150 topic groups. No exact ASCII-normalized duplicates found. Group split seed 42: 90/30/30 topics and 1,348/453/439 records in train/validation/test. Vocabulary and IDF fit on training only. 10,000 ASCII word features, sublinear TF, L2 normalization, 120 batch gradient iterations, L2 penalty 0.001. No paid API, GPU rental or server inference.

Threshold 0.47677173837318837 is selected only from validation scores, maximizing recall under empirical validation FPR ≤5%. Validation FPR: 10/210=4.76%. This does not guarantee future FPR. No test-dependent tuning occurred. Test vocabulary-token-set Jaccard >=0.8 against train/validation found zero overlapping records; this is not a semantic leakage audit.

## Actual internal test results
439 records, 219 AI-labelled / 220 human-labelled:
- TP 200; FP 6; TN 214; FN 19.
- Accuracy 94.31%; precision 97.09%; recall 91.32%; F1 94.12%.
- FPR 2.73%; FNR 8.68%.
- ROC AUC 0.99126; average precision 0.99141.
- Brier 0.17795; 10-bin ECE 0.31630: calibration is poor. Scores are NOT calibrated probabilities.

Machine-readable reports: models/evaluation-v1.json and models/diagnostics-v1.json. These results describe this dataset and split only. Source differences (Wikipedia entities versus Gemma general explanations) may explain discrimination rather than transferable AI-writing signals. Only two closely related Gemma variants are present; no unseen generator, adversarial, human-edited AI, independent academic or domain-shift evaluation has been performed.

## Runtime and interpretation
Model JSON is bundled in WASM, eliminating runtime model downloads and partial-cache failures for this small baseline. This intentionally trades lazy downloading for simpler immutable deployment; full offline support is not claimed. Hash is recorded in the manifest; deployment assets use generated integrity metadata. User documents stay in browser memory.

UI runs inference only after explicit research-mode selection. Paragraphs with 25–200 words qualify (conservative range around this corpus's observed 21–199 words); others remain visible and are excluded. No known vocabulary causes abstention. Low partial vocabulary coverage is displayed but not a validated confidence estimate. Flagged percentage is word-weighted over eligible paragraphs, not probability of AI authorship. A threshold-negative segment is not proven human. Scores and percentages must not be used to rank student honesty.

Browser CPU validation, model size and current checks are recorded in PROGRESS. Burn/WebGPU are not used: this sparse linear model needs only native arithmetic. No transformer quality claim is made. Peak browser memory and cross-browser support remain unmeasured.
