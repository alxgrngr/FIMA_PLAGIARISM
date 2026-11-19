"""Train a private Rust-compatible pilot on reviewed journals and topic-matched AI."""

import argparse
from collections import Counter
import hashlib
import json
import math
from pathlib import Path

from sklearn.linear_model import LogisticRegression

from baseline import load, predict, tokens, vector
from candidate import paragraphs, sparse_features

THRESHOLD = 0.5
REGULARIZATION = 3.0
BASELINE = Path("models/detectai-topic-v1.json")


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_journals(path):
    documents = []
    groups = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        row = json.loads(line)
        if (not isinstance(row.get("text"), str) or not row["text"].strip()
                or not isinstance(row.get("group"), str) or not row["group"].strip()
                or not isinstance(row.get("source"), str) or not row["source"].strip()):
            raise ValueError("Each journal needs original paragraph text, an article group and provenance.")
        if row["group"] in groups:
            raise ValueError("Combine each journal's paragraphs in one record.")
        groups.add(row["group"])
        documents.append({**row, "label": 0})
    if len(groups) < 3:
        raise ValueError("At least three journal groups are needed for this pilot.")
    return documents


def check_groups(journals, generated):
    if any(row["label"] != 1 for row in generated):
        raise ValueError("The generated dataset must contain AI labels only.")
    groups = {row["group"] for row in journals}
    if groups != {row["group"] for row in generated}:
        raise ValueError("Generated samples need exactly the same article group IDs as the journals.")
    selected = list(paragraphs(journals + generated))
    if {(r["group"], r["label"]) for r in selected} != {(g, y) for g in groups for y in (0, 1)}:
        raise ValueError("Every article group needs eligible journal and AI paragraphs.")
    seen = set()
    for row in selected:
        key = tuple(tokens(row["text"]))
        if not key or key in seen:
            raise ValueError("Review and remove empty or exact duplicate normalized paragraphs.")
        seen.add(key)
    return selected, groups


def train(rows):
    df = Counter(term for row in rows for term in set(tokens(row["text"])))
    terms = sorted(df, key=lambda term: (-df[term], term))[:10_000]
    vocabulary = {term: i for i, term in enumerate(terms)}
    idf = [1 + math.log((1 + len(rows)) / (1 + df[term])) for term in terms]
    matrix = sparse_features(rows, vocabulary, idf)
    counts = Counter((r["group"], r["label"]) for r in rows)
    # Each article/class cell has the same total weight, even if one PDF is longer.
    weight = [len(rows) / (len(counts) * counts[r["group"], r["label"]]) for r in rows]
    model = LogisticRegression(C=REGULARIZATION, solver="liblinear", random_state=42)
    model.fit(matrix, [r["label"] for r in rows], sample_weight=weight)
    return {
        "schema_version": 1, "status": "research_only", "tokenizer": "ascii_lower_v1",
        "vocabulary": vocabulary, "idf": idf, "weights": model.coef_[0].tolist(),
        "bias": float(model.intercept_[0]), "threshold": THRESHOLD, "calibrated": False,
    }


def audit(rows, model):
    counts = {"journal_paragraphs": 0, "journal_flagged": 0, "journal_abstained": 0,
              "ai_paragraphs": 0, "ai_flagged": 0, "ai_abstained": 0}
    for row in rows:
        kind = "ai" if row["label"] else "journal"
        counts[kind + "_paragraphs"] += 1
        features = vector(row["text"], model["vocabulary"], model["idf"])
        if not features:
            counts[kind + "_abstained"] += 1
        elif predict(features, model["weights"], model["bias"]) >= model["threshold"]:
            counts[kind + "_flagged"] += 1
    return counts


def total(folds, model):
    return {key: sum(fold[model][key] for fold in folds) for key in folds[0][model]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("journals", type=Path, help="Private JSONL, one paragraph-delimited record per journal")
    parser.add_argument("generated", type=Path, help="Reviewed generated JSONL with paired article group IDs")
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.out.exists():
        parser.error("--out must be a new directory.")
    journals = read_journals(args.journals)
    generated = load(args.generated)
    rows, groups = check_groups(journals, generated)
    baseline = json.loads(BASELINE.read_text(encoding="utf-8"))
    folds = []
    for group in sorted(groups):
        development = [row for row in rows if row["group"] != group]
        holdout = [row for row in rows if row["group"] == group]
        folds.append({"article_group": group, "live_v1": audit(holdout, baseline),
                      "pilot": audit(holdout, train(development))})
    final = train(rows)
    final_bytes = json.dumps(final, sort_keys=True, separators=(",", ":")).encode()
    report = {
        "articles": len(groups), "model_threshold": THRESHOLD, "regularization_C": REGULARIZATION,
        "selected_journal_paragraphs": sum(len(r["text"].splitlines()) for r in journals),
        "generator_sources": sorted({row["source"] for row in generated}),
        "eligible_by_article_and_source": {
            group: dict(sorted(Counter(r["source"] for r in rows if r["group"] == group).items()))
            for group in sorted(groups)
        },
        "sha256": {"journals": digest(args.journals), "generated": digest(args.generated),
                   "live_v1": digest(BASELINE), "pilot": hashlib.sha256(final_bytes).hexdigest()},
        "leave_one_article_out": folds,
        "held_out_totals": {model: total(folds, model) for model in ("live_v1", "pilot")},
        "limitations": [
            "Journal text is a human-authorship proxy. Three groups do not establish detector quality.",
            "Each fold holds out an article and its paired AI topic; the same generators appear across folds.",
            "The exported model is fit on all groups. Fold results concern the training procedure.",
            "Fixed threshold, no calibration, no student-report or independent-generator validation.",
            "Private research pilot; usage rights for some journal texts remain unconfirmed.",
        ],
    }
    args.out.mkdir(parents=True, exist_ok=False)
    (args.out / "journal-pilot.json").write_bytes(final_bytes)
    (args.out / "evaluation.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report["held_out_totals"], indent=2))


if __name__ == "__main__":
    main()
