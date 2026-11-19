# Reproducible research baseline

This is training/evaluation tooling, **not a validated general-purpose detector**. Python standard library only; production frontend remains Rust/WASM. A sparse TF-IDF/logistic-regression baseline is transparent and small enough to explore CPU browser inference, and topic-v1 research weights are now integrated behind an explicit opt-in.

## Input and rights review
Provide UTF-8 JSONL records with `text`, integer `label` (0 human, 1 AI), `group`, `source`, `license`. Review the actual license, provenance and rights before use; nonempty metadata is not legal verification. Keep local datasets in ignored `datasets/local/`; do not commit corpus content. Put source/AI rewrites of the same document, near duplicates and related documents into the same group. Exact normalized duplicates are rejected; semantic near duplicates require external audit.

```
python3 -m unittest discover -s training -v
python3 training/baseline.py datasets/local/reviewed.jsonl --out training/runs/experiment-001
```

The output directory must not exist. Outputs include model weights, vocabulary/IDF, dataset/model SHA-256, actual evaluation metrics and split membership. Seed 42, 60/20/20 group split, training-only vocabulary and IDF. Batch gradient descent is deterministic. Threshold maximizes validation recall subject to empirical FPR <=5%; the finite validation sample provides no guarantee of population FPR. Test is never used for threshold selection. Precision/F1 may be null when undefined. Scores are uncalibrated; do not call them calibrated probabilities.

## Evidence and limits
Five software unit tests pass. Synthetic fixtures are used solely for algorithm/metric/splitting checks and are not evidence of detector accuracy. The initial software-only checks preceded the real topic-v1 run documented below. No academic/domain-shift evaluation, confidence intervals, ROC/PR AUC, baseline comparison or production approval exists. Tokenizer intentionally retains ASCII letters only; this must be revisited before multilingual use. At least 15 groups is an execution guard, not an adequate dataset-size claim. Memory is proportional to corpus/vocabulary and dense gradients; benchmark before large datasets.

## Native Rust inference
`cargo run --locked --example score -- MODEL.json TEXT.txt` loads the research JSON format and returns an uncalibrated score. All-unknown vocabulary abstains. Native Rust math was compared to Python on three synthetic test inputs within 1e-12. This verifies numerical plumbing, not detector quality. The public UI deliberately does not expose loading unvalidated research weights.

## Actual Indonesian experiment (topic-v1)
Download the pinned public `train.csv` from `https://huggingface.co/datasets/shouwiku/detectai-indonesian/resolve/7ba07fffc32ec81b0d6e660a1ebfea9ec9400fc0/train.csv` into ignored `datasets/local/detectai.csv`. Review docs/MODEL_CARD.md provenance first. Run `python3 training/prepare_detectai.py` (checks SHA-256), then `python3 training/baseline.py datasets/local/prepared.jsonl --out training/runs/detectai-topic-v1` using a fresh output directory. `python3 training/evaluate_detectai.py` computes locked-test diagnostics against the committed model. These data are not to be committed. This model has real internal test metrics but remains research-only.

## Paragraph candidate v2 on Windows

Use Python 3.12 and PowerShell from the repository root. This experiment needs no GPU or paid API. The pinned corpus used here is public. Read the provenance section of docs/MODEL_CARD.md before using it.

```powershell
py -3.12 -m venv .venv
.\.venv\Scripts\python.exe -m pip install -r training/requirements-candidate.txt
New-Item -ItemType Directory -Force datasets/local | Out-Null
curl.exe -fL "https://huggingface.co/datasets/shouwiku/detectai-indonesian/resolve/7ba07fffc32ec81b0d6e660a1ebfea9ec9400fc0/train.csv" -o datasets/local/detectai.csv
.\.venv\Scripts\python.exe training/prepare_detectai.py
.\.venv\Scripts\python.exe training/candidate.py datasets/local/prepared.jsonl --out training/runs/paragraph-v2-laptop
```

The runner splits by topic before extracting paragraphs and discards exact normalized paragraph overlap between splits. It trains TF-IDF logistic regression on paragraphs of 25–200 words, compares seven regularization settings on validation only, and evaluates the selected candidate and live model on the same topic-held-out paragraphs. Split membership is unchanged from topic-v1, so the paragraph results revisit the original source and are an internal comparison. Scores remain uncalibrated.

The trained candidate and its aggregate report are in `models/experimental/`. On 836 held-out paragraphs the candidate caught 401/445 AI-labelled paragraphs versus 358/445 for the live model, while incorrectly flagging 22/391 human-labelled paragraphs versus 16/391. The candidate is more sensitive on this test and more often flagged human paragraphs incorrectly at its chosen threshold. The deployed model is still topic-v1.

To check suitability for Perilaku Organisasi, add `--domain-test datasets/local/praktikum-test.jsonl` and choose a fresh output folder. The file uses the same input fields as above. Use consented, anonymized student text with verified authorship and AI samples from multiple tools answering comparable prompts. Keep each prompt or assignment and related versions together. The optional test file stays local and is evaluated only after training. Use a separate set for any later model tuning. Review false positives on held-out human reports, recall across generators, and the number of paragraphs excluded before choosing a production model.

## Word and character experiment with independent generators

Python 3.12 and `training/requirements-candidate.txt` also support this CPU experiment. After preparing the pinned corpus above, run:

```powershell
.\.venv\Scripts\python.exe training/robust_candidate.py datasets/local/prepared.jsonl --out training/runs/word-ngram-v3
```

The script compares word unigrams and bigrams, character n-grams, and their combination. It selects the classifier and decision threshold on the validation split before reading any independent test. The model file is a local scikit-learn artifact. Only load Joblib files from training runs you trust. This is a laptop experiment and isn't loaded by the website.

To reproduce the AI-only test, install Node.js 20 or newer and use `npm install --prefix training/runs/llama node-llama-cpp@3.22.3`. Download the Q4_K_M GGUF directly from a model repository listed in the evaluation report's `generation` metadata. Run the following with the actual file path and exact source string from the report:

```powershell
node training/generate_stress.mjs training/runs/llama path/to/weights.gguf "GENERATOR@REVISION" datasets/local/stress-one.jsonl
```

Review the generated responses for language, topic and prompt echo before combining them into a single UTF-8 JSONL file. Then rerun the experiment with a fresh output directory and `--ai-stress-test datasets/local/stress-reviewed.jsonl`. Sampled outputs can differ between platforms; keep model hashes, generator versions, prompts and accepted sample IDs with any results. Use `--domain-test datasets/local/praktikum-test.jsonl` to also measure false positives on independently reviewed Bahasa Indonesia reports and AI examples. Related versions must share group IDs, with distinct groups across development and test. The training command does not send reports to a service.

## Private pilot from the three supplied journals

The CPU pipeline `training/journal_pilot.py` fits a compact Rust-compatible research model from the three journal sources alone, with AI samples paired to their topics. The archived Wikipedia/Gemma weights are read only as a comparison. There is no call to a paid service.

- Journal JSONL: one record per article with `text` (one original paragraph per line), `source` (publisher DOI URL) and `group` (DOI). See the earlier journal audit report for the exact paragraph spans selected from each user-supplied PDF. Keep these excerpts under ignored `datasets/local/`.
- Generated JSONL: use the optional fifth argument to `training/generate_stress.mjs`, a JSON array of `{group,prompt}` objects with article DOI as the group. Generate independently from topic descriptions, without passing the article text into the generators. Review relevance and remove headings or irrelevant sections before fitting.
- Train with the Python environment documented above.

```powershell
.\.venv\Scripts\python.exe training/journal_pilot.py datasets/local/journal-pilot/journals.jsonl datasets/local/journal-pilot/generated-reviewed.jsonl --out training/runs/journal-three-pilot
```

For each evaluation fold, the pipeline holds out one article and all AI text paired with its topic. The final JSON model uses all three documents and the generated examples. Its fold report describes the training procedure, with only three independent groups and no validation on student writing or unseen generators. Keep the output private under ignored `training/runs/`, especially while reuse licensing is unconfirmed for two of the PDFs.
