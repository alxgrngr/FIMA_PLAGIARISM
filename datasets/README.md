# Dataset sourcing audit

No corpus is bundled and no dataset has been approved for model training.

## M4 candidate
Source: https://github.com/mbzuai-nlp/M4
Inspected README.md, data/README.md and Git tree on 2026-11-16.
- Includes `data/id-newspaper_chatGPT.jsonl`, with documented human_text/machine_text/model/source/source_ID fields.
- This is Indonesian newspaper material; it does not establish academic-domain coverage.
- No LICENSE file appeared in the inspected repository tree. Documentation inspected does not establish downstream training/redistribution rights.
- Therefore no candidate samples were used for training or redistributed.
- Paper reports cross-generator/domain generalization difficulties. It is not evidence that this application's future model works.

Hugging Face dataset discovery requests returned HTTP 403 from the network proxy. Additional domains have been saved in the environment network draft; runtime access is pending user application of the settings. Data provenance, license, grouping, duplicates, class balance, human-written academic test cases and independent test splits remain required before a production model can be evaluated.

## Research recheck, 2026-11-16
Hugging Face access now succeeds. Searches for Indonesian detection corpora did not establish an approved corpus. The `d0rj/SemEval2024-task8` card identifies itself as an unofficial mirror and claims Apache-2.0. Attempting to inspect the official upstream Git repository returned HTTP 403 with an explicit GitHub DMCA takedown notice (`github/dmca`, 2026/09/2026-09-14-wikihow.md). The mirror's license label therefore does not resolve upstream rights; no training data was downloaded from it. This does not establish that every source within M4 is infringing, but prevents blanket approval of the collection.

`browndw/human-ai-parallel-corpus` documents an English corpus with MIT metadata, explicit human continuation / LLM continuation pairs and mixed source genres. It is not Indonesian validation data; no model for Indonesian users was trained on it. Further source-specific rights and grouping review would be needed before a separate English experiment.

## Selected research corpus: DETECTAI
A broader Indonesian dataset search found `shouwiku/detectai-indonesian`, revision `7ba07fffc32ec81b0d6e660a1ebfea9ec9400fc0`. Its publisher declares MIT and documents Wikipedia human-labelled text / Gemma AI-labelled text. CSV contains 2,240 records, source, topic, page_title and model metadata. Unlike the earlier unapproved SemEval mirror, this is a separately documented collection; see docs/MODEL_CARD.md for publisher citation and source-rights caveats. Topic groups are retained across classes. Dataset stays in ignored datasets/local; only learned coefficients and aggregate evaluation artifacts are committed. Publisher labels are not independently verified. Wikipedia source membership is confounded with the human label; no general detector claim follows from internal accuracy.
