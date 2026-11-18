"""Train a paragraph-level Rust-compatible candidate and compare with the live model."""

import argparse
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import unicodedata

from scipy.sparse import csr_matrix
from sklearn.linear_model import LogisticRegression

from baseline import load, metrics, predict, split, tokens, vector

REGULARIZATION = (0.1, 0.3, 1.0, 3.0, 10.0, 30.0, 100.0)
FPR_TARGET = 0.05


def paragraphs(documents):
    for doc in documents:
        for line in doc["text"].split("\n"):
            line = unicodedata.normalize("NFC", line.strip())
            if 25 <= len(line.split()) <= 200:
                yield {**doc, "text": line}


def deduplicate(parts):
    """Drop cross-split duplicate paragraphs from the derived data."""
    memberships = {}
    for index, part in enumerate(parts):
        for row in part:
            key = tuple(tokens(row["text"]))
            memberships.setdefault(key, set()).add((index, row["label"]))
    cleaned = []
    for part in parts:
        seen = set()
        keep = []
        for row in part:
            key = tuple(tokens(row["text"]))
            if key and len(memberships[key]) == 1 and key not in seen:
                seen.add(key)
                keep.append(row)
        cleaned.append(keep)
    return cleaned


def sparse_features(rows, vocabulary, idf):
    indices, values, pointers = [], [], [0]
    for row in rows:
        features = vector(row["text"], vocabulary, idf)
        for index, value in sorted(features.items()):
            indices.append(index)
            values.append(value)
        pointers.append(len(indices))
    return csr_matrix((values, indices, pointers), shape=(len(rows), len(vocabulary)))


def score(rows, model):
    labels, scores = [], []
    abstained = Counter()
    for row in rows:
        features = vector(row["text"], model["vocabulary"], model["idf"])
        if features:
            labels.append(row["label"])
            scores.append(predict(features, model["weights"], model["bias"]))
        else:
            abstained[row["label"]] += 1
    if set(labels) != {0, 1}:
        raise ValueError("Each evaluation split must have scoreable paragraphs from both classes.")
    return labels, scores, abstained


def summarize(labels, scores, abstained, threshold):
    return {**metrics(labels, scores, threshold),
            "threshold": threshold, "eligible_paragraphs": len(labels) + sum(abstained.values()),
            "abstained_human": abstained[0], "abstained_ai": abstained[1]}


def threshold_for(labels, scores, abstained):
    human_scores = sorted((p for y, p in zip(labels, scores) if y == 0), reverse=True)
    allowable = math.floor(FPR_TARGET * len(human_scores))
    cutoff = human_scores[allowable]
    for threshold in sorted(set(p for p in scores if p > cutoff) | {1.0}):
        report = summarize(labels, scores, abstained, threshold)
        if report["false_positive_rate"] <= FPR_TARGET:
            return threshold, report
    raise ValueError("No threshold meets the validation false-positive target.")


def fit(training, validation):
    df = Counter(term for row in training for term in set(tokens(row["text"])))
    terms = sorted(df, key=lambda term: (-df[term], term))[:10_000]
    vocabulary = {term: index for index, term in enumerate(terms)}
    idf = [math.log((1 + len(training)) / (1 + df[term])) + 1 for term in terms]
    features = sparse_features(training, vocabulary, idf)
    labels = [row["label"] for row in training]
    fits = []
    for c in REGULARIZATION:
        fitted = LogisticRegression(C=c, solver="liblinear", max_iter=1000, random_state=42)
        fitted.fit(features, labels)
        model = {
            "schema_version": 1, "status": "research_only", "tokenizer": "ascii_lower_v1",
            "vocabulary": vocabulary, "idf": idf, "weights": fitted.coef_[0].tolist(),
            "bias": float(fitted.intercept_[0]), "calibrated": False,
        }
        model["threshold"], result = threshold_for(*score(validation, model))
        fits.append((c, model, result))
    return max(fits, key=lambda fit: (fit[2]["recall"], -fit[2]["false_positive_rate"], -fit[0]))


def compare(rows, baseline, candidate):
    return {label: summarize(*score(rows, model), model["threshold"])
            for label, model in (("live", baseline), ("candidate", candidate))}


def independent_paragraphs(path, development_documents, development_paragraphs):
    documents = load(path)
    if {r["group"] for r in documents} & {r["group"] for r in development_documents}:
        raise ValueError("Independent test data must not reuse development group IDs.")
    independent = list(paragraphs(documents))
    originals = {tuple(tokens(row["text"])) for row in development_paragraphs}
    if any(tuple(tokens(row["text"])) in originals for row in independent):
        raise ValueError("Paragraph overlap between development and the independent test.")
    cleaned, = deduplicate([independent])
    return cleaned


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dataset", type=Path)
    parser.add_argument("--baseline", type=Path, default=Path("models/detectai-topic-v1.json"))
    parser.add_argument("--domain-test", type=Path, help="Independent, locally reviewed labelled JSONL")
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.out.exists():
        parser.error("--out must name a new directory.")
    documents = load(args.dataset)
    parts = [list(paragraphs(part)) for part in split(documents)]
    training, validation, test = deduplicate(parts)
    if any({row["label"] for row in part} != {0, 1} for part in (training, validation, test)):
        raise ValueError("Each split needs eligible paragraphs from both classes.")
    baseline = json.loads(args.baseline.read_text(encoding="utf-8"))
    regularization, model, validation_report = fit(training, validation)
    report = {
        "dataset_sha256": hashlib.sha256(args.dataset.read_bytes()).hexdigest(),
        "baseline_sha256": hashlib.sha256(args.baseline.read_bytes()).hexdigest(),
        "sources": sorted({row["source"] for row in documents}),
        "split_seed": 42, "paragraph_sizes": [len(part) for part in (training, validation, test)],
        "excluded_duplicate_paragraphs": sum(map(len, parts)) - len(training) - len(validation) - len(test),
        "selected_C": regularization, "validation_fpr_limit": FPR_TARGET,
        "candidate_validation": validation_report,
        "internal_test": compare(test, baseline, model),
        "domain_test": None,
    }
    if args.domain_test:
        domain_paragraphs = independent_paragraphs(
            args.domain_test, documents, (r for part in parts for r in part),
        )
        report["domain_test"] = compare(domain_paragraphs, baseline, model)
        report["domain_test_sha256"] = hashlib.sha256(args.domain_test.read_bytes()).hexdigest()
    data = json.dumps(model, sort_keys=True, separators=(",", ":")).encode()
    report["candidate_sha256"] = hashlib.sha256(data).hexdigest()
    args.out.mkdir(parents=True, exist_ok=False)
    (args.out / "candidate.json").write_bytes(data)
    (args.out / "comparison.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report["internal_test"], indent=2))


if __name__ == "__main__":
    main()
