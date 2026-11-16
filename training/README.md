# Reproducible research baseline

This is training/evaluation tooling, **not a trained detector**. Python standard library only; production frontend remains Rust/WASM. A sparse TF-IDF/logistic-regression baseline is transparent and small enough to explore CPU browser inference, but no weights have been integrated into the website.

## Input and rights review
Provide UTF-8 JSONL records with `text`, integer `label` (0 human, 1 AI), `group`, `source`, `license`. Review the actual license, provenance and rights before use; nonempty metadata is not legal verification. Keep local datasets in ignored `datasets/local/`; do not commit corpus content. Put source/AI rewrites of the same document, near duplicates and related documents into the same group. Exact normalized duplicates are rejected; semantic near duplicates require external audit.

```
python3 -m unittest discover -s training -v
python3 training/baseline.py datasets/local/reviewed.jsonl --out training/runs/experiment-001
```

The output directory must not exist. Outputs include model weights, vocabulary/IDF, dataset/model SHA-256, actual evaluation metrics and split membership. Seed 42, 60/20/20 group split, training-only vocabulary and IDF. Batch gradient descent is deterministic. Threshold maximizes validation recall subject to empirical FPR <=5%; the finite validation sample provides no guarantee of population FPR. Test is never used for threshold selection. Precision/F1 may be null when undefined. Scores are uncalibrated; do not call them calibrated probabilities.

## Evidence and limits
Five software unit tests pass. Synthetic fixtures are used solely for algorithm/metric/splitting checks and are not evidence of detector accuracy. No real-data training run has occurred. No academic/domain-shift evaluation, confidence intervals, ROC/PR AUC, baseline comparison or production approval exists. Tokenizer intentionally retains ASCII letters only; this must be revisited before multilingual use. At least 15 groups is an execution guard, not an adequate dataset-size claim. Memory is proportional to corpus/vocabulary and dense gradients; benchmark before large datasets.
