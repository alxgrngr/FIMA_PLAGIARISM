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
