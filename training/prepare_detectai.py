"""Prepare pinned DETECTAI corpus; local-only text outputs, no redistribution."""
import csv, json, hashlib
from pathlib import Path
from baseline import tokens
source=Path('datasets/local/detectai.csv')
assert hashlib.sha256(source.read_bytes()).hexdigest() == '013813fc408fae24cb5157fa4555721c325287f8020ac10b9bafc61b7190ef65', 'Dataset checksum mismatch'
rows=list(csv.DictReader(source.open(encoding='utf-8-sig')))
seen=set(); output=[]; removed=0
for r in rows:
    key=' '.join(tokens(r['text']))
    if key in seen: removed+=1; continue
    seen.add(key)
    if not r['topic'].strip(): raise ValueError('Missing topic group')
    output.append(dict(text=r['text'],label=int(r['label']),group=r['topic'].strip().lower(),source='shouwiku/detectai-indonesian@7ba07fffc32ec81b0d6e660a1ebfea9ec9400fc0',license='MIT declared by publisher; Wikipedia source attribution/CC-BY-SA obligations retained'))
Path('datasets/local/prepared.jsonl').write_text('\n'.join(json.dumps(r,ensure_ascii=False) for r in output))
print(json.dumps(dict(source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),rows=len(output),removed_duplicates=removed,groups=len({r['group'] for r in output}))))
