"""Train an experimental word/character detector and audit independent AI samples."""

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path

import joblib
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.linear_model import LogisticRegression
from sklearn.pipeline import FeatureUnion, Pipeline

from baseline import load, predict, split, vector
from candidate import deduplicate, independent_paragraphs, paragraphs, score, summarize, threshold_for

ENCODERS = ("word", "character", "combined")
REGULARIZATION = (1.0, 10.0, 30.0)


def encoder(kind):
    word = TfidfVectorizer(
        token_pattern=r"(?u)\b[a-z]+\b", ngram_range=(1, 2), min_df=2,
        max_features=12_000, sublinear_tf=True,
    )
    character = TfidfVectorizer(
        analyzer="char_wb", ngram_range=(3, 5), min_df=3,
        max_features=16_000, sublinear_tf=True,
    )
    if kind == "combined":
        return FeatureUnion([("word", word), ("character", character)])
    return word if kind == "word" else character


def fit(training, validation):
    train_texts = [r["text"] for r in training]
    train_labels = [r["label"] for r in training]
    validation_labels = [r["label"] for r in validation]
    validation_texts = [r["text"] for r in validation]
    trials = []
    leader = None
    for kind in ENCODERS:
        features = encoder(kind)
        training_matrix = features.fit_transform(train_texts)
        validation_matrix = features.transform(validation_texts)
        covered = validation_matrix.getnnz(axis=1) > 0
        scored_labels = [label for label, usable in zip(validation_labels, covered) if usable]
        abstained = Counter(label for label, usable in zip(validation_labels, covered) if not usable)
        if set(scored_labels) != {0, 1}:
            raise ValueError("Validation needs scoreable paragraphs from both classes.")
        for c in REGULARIZATION:
            lr = LogisticRegression(C=c, solver="liblinear", max_iter=1000, random_state=42)
            lr.fit(training_matrix, train_labels)
            scores = lr.predict_proba(validation_matrix[covered])[:, 1].tolist()
            threshold, val = threshold_for(scored_labels, scores, abstained)
            trials.append({"features": kind, "C": c, "validation": val})
            rank = (val["recall"], -val["false_positive_rate"], -training_matrix.shape[1], -c)
            if leader is None or rank > leader[0]:
                leader = (rank, kind, c, Pipeline([("features", features), ("classifier", lr)]),
                          threshold, training_matrix.shape[1])
    return leader[1:], trials


def evaluate_ai(documents, eligible, model, threshold, baselines):
    """AI-only stress tests measure misses, never false-positive rate."""
    results = {}
    for generator, count in Counter(row["source"] for row in documents).items():
        items = [row for row in eligible if row["source"] == generator]
        if not items:
            results[generator] = {"generated_examples": count, "eligible_paragraphs": 0}
            continue
        matrix, covered = predictor_inputs(items, model)
        scores = (model.named_steps["classifier"].predict_proba(matrix[covered])[:, 1]
                  if covered.any() else [])
        old = {}
        for name, baseline in baselines.items():
            matched = 0
            abstained = 0
            # Evaluate each paragraph without imposing score()'s two-class requirement.
            for row in items:
                values = vector(row["text"], baseline["vocabulary"], baseline["idf"])
                if not values:
                    abstained += 1
                elif predict(values, baseline["weights"], baseline["bias"]) >= baseline["threshold"]:
                    matched += 1
            old[name] = {"flagged": matched, "abstained": abstained}
        results[generator] = {
            "generated_examples": count, "eligible_paragraphs": len(items),
            "candidate_flagged": sum(1 for value in scores if value >= threshold),
            "candidate_abstained": len(items) - int(covered.sum()), **old,
        }
    return results


def predictor_inputs(rows, model):
    matrix = model.named_steps["features"].transform([r["text"] for r in rows])
    return matrix, matrix.getnnz(axis=1) > 0


def compare_all(rows, model, threshold, baselines):
    if not rows:
        raise ValueError("No eligible test paragraphs.")
    matrix, covered = predictor_inputs(rows, model)
    labels = [r["label"] for r, usable in zip(rows, covered) if usable]
    if set(labels) != {0, 1}:
        raise ValueError("Test needs scoreable paragraphs from both classes.")
    abstained = Counter(r["label"] for r, usable in zip(rows, covered) if not usable)
    scores = model.named_steps["classifier"].predict_proba(matrix[covered])[:, 1].tolist()
    return {**{name: summarize(*score(rows, baseline), baseline["threshold"])
               for name, baseline in baselines.items()},
            "candidate": summarize(labels, scores, abstained, threshold)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("dataset", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--ai-stress-test", type=Path, help="Locally generated JSONL, never used in training")
    parser.add_argument("--domain-test", type=Path, help="Independent, locally reviewed human/AI JSONL")
    args = parser.parse_args()
    if args.out.exists():
        parser.error("--out must name a new directory.")
    documents = load(args.dataset)
    parts = [list(paragraphs(part)) for part in split(documents)]
    training, validation, test = deduplicate(parts)
    (kind, c, model, threshold, size), trials = fit(training, validation)
    baselines = {
        "live": json.loads(Path("models/detectai-topic-v1.json").read_text(encoding="utf-8")),
        "paragraph_v2": json.loads(Path("models/experimental/detectai-paragraph-v2.json").read_text(encoding="utf-8")),
    }
    report = {
        "dataset_sha256": hashlib.sha256(args.dataset.read_bytes()).hexdigest(),
        "selected": {"features": kind, "C": c, "threshold": threshold, "feature_count": size},
        "paragraph_counts": list(map(len, (training, validation, test))),
        "validation_trials": trials,
        "internal_test": compare_all(test, model, threshold, baselines),
        "ai_stress_test": None,
        "domain_test": None,
        "limitations": [
            "Uncalibrated scores. Internal test revisits the Wikipedia/Gemma dataset.",
            "AI-only stress test cannot estimate a false-positive rate on student reports.",
            "Experimental Python model, not bundled in production.",
        ],
    }
    if args.ai_stress_test:
        stress = load(args.ai_stress_test)
        if any(row["label"] != 1 for row in stress):
            raise ValueError("AI stress test must contain AI-labelled samples only.")
        eligible = independent_paragraphs(
            args.ai_stress_test, documents, (r for part in parts for r in part),
        )
        report["ai_stress_test"] = evaluate_ai(stress, eligible, model, threshold, baselines)
        report["ai_stress_test_sha256"] = hashlib.sha256(args.ai_stress_test.read_bytes()).hexdigest()
    if args.domain_test:
        domain = independent_paragraphs(
            args.domain_test, documents, (r for part in parts for r in part),
        )
        report["domain_test"] = compare_all(domain, model, threshold, baselines)
        report["domain_test_sha256"] = hashlib.sha256(args.domain_test.read_bytes()).hexdigest()
    else:
        report["limitations"].append("No independent domain test with human-authored reports was supplied.")
    args.out.mkdir(parents=True, exist_ok=False)
    joblib.dump({"model": model, "threshold": threshold}, args.out / "experimental.joblib", compress=3)
    (args.out / "evaluation.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: report[key] for key in (
        "selected", "internal_test", "ai_stress_test", "domain_test",
    )}, indent=2))


if __name__ == "__main__":
    main()
