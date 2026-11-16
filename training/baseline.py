"""Reproducible research baseline. Standard library only; no network access.
Input JSONL: text, label (0 human / 1 AI), group, source, license.
No corpus is bundled. All related versions of a document must share a group.
"""
import argparse
import collections
import hashlib
import json
import math
import random
import re
from pathlib import Path


def tokens(text):
    # Deliberately ASCII vocabulary to allow exact Rust parity; Indonesian
    # diacritics/non-ASCII tokens are excluded, an explicit baseline limitation.
    return re.findall(r'[a-z]+', text.lower())


def load(path):
    rows = [json.loads(line) for line in Path(path).read_text().splitlines() if line.strip()]
    seen = {}
    for row in rows:
        if not isinstance(row.get('text'), str) or not tokens(row['text']):
            raise ValueError('Missing usable text')
        if type(row.get('label')) is not int or row['label'] not in (0, 1):
            raise ValueError('Label must be 0 or 1')
        for field in ('group', 'source', 'license'):
            if not isinstance(row.get(field), str) or not row[field].strip():
                raise ValueError('Missing provenance: ' + field)
        fingerprint = ' '.join(tokens(row['text']))
        if fingerprint in seen:
            raise ValueError('Duplicate normalized text; deduplicate and review provenance first')
        seen[fingerprint] = row['label']
    return rows


def split(rows):
    groups = sorted({r['group'] for r in rows})
    random.Random(42).shuffle(groups)
    n = len(groups)
    if n < 15:
        raise ValueError('At least 15 independent groups required for pipeline execution; this is not a scientific sample-size guarantee')
    train = set(groups[:int(n*.6)])
    valid = set(groups[int(n*.6):int(n*.8)])
    parts = [[r for r in rows if (r['group'] in train if i == 0 else r['group'] in valid if i == 1 else r['group'] not in train | valid)] for i in range(3)]
    if any({r['label'] for r in p} != {0, 1} for p in parts):
        raise ValueError('Every split must contain both labels; supply more independent groups')
    return parts


def vector(text, vocabulary, idf):
    counts = collections.Counter(tokens(text))
    v = {vocabulary[t]: (1+math.log(c))*idf[vocabulary[t]] for t,c in counts.items() if t in vocabulary}
    norm = math.sqrt(sum(x*x for x in v.values()))
    return {i:x/norm for i,x in v.items()} if norm else {}


def sigmoid(x):
    return 1/(1+math.exp(-max(-60, min(60, x))))


def predict(v, w, b):
    return sigmoid(b + sum(w[i]*x for i,x in v.items()))


def metrics(labels, scores, threshold):
    tp=sum(y==1 and p>=threshold for y,p in zip(labels,scores))
    fp=sum(y==0 and p>=threshold for y,p in zip(labels,scores))
    tn=sum(y==0 and p<threshold for y,p in zip(labels,scores))
    fn=sum(y==1 and p<threshold for y,p in zip(labels,scores))
    div=lambda a,b: a/b if b else None
    return dict(n=len(labels), tp=tp, fp=fp, tn=tn, fn=fn, accuracy=div(tp+tn,len(labels)), precision=div(tp,tp+fp), recall=div(tp,tp+fn), f1=div(2*tp,2*tp+fp+fn), false_positive_rate=div(fp,fp+tn), false_negative_rate=div(fn,fn+tp), brier=sum((p-y)**2 for y,p in zip(labels,scores))/len(labels))


def train(rows, max_features=10000, epochs=120):
    training, validation, test = split(rows)
    df=collections.Counter(t for r in training for t in set(tokens(r['text'])))
    vocab={t:i for i,(t,_) in enumerate(sorted(df.items(),key=lambda p:(-p[1],p[0]))[:max_features])}
    idf=[math.log((1+len(training))/(1+df[t]))+1 for t in vocab]
    xs=[vector(r['text'],vocab,idf) for r in training]
    w=[0.0]*len(vocab); b=0.0
    for _ in range(epochs):
        grad=[0.0]*len(w); gb=0.0
        for r,x in zip(training,xs):
            error=predict(x,w,b)-r['label']; gb+=error
            for i,v in x.items(): grad[i]+=error*v
        b-=gb/len(xs)
        w=[v-(g/len(xs)+.001*v) for v,g in zip(w,grad)]
    valid_scores=[predict(vector(r['text'],vocab,idf),w,b) for r in validation]
    # Threshold selection uses validation only: maximize recall subject to <=5%
    # empirical FPR. This does not establish a population FPR guarantee.
    candidates=sorted(set(valid_scores+[1.0]))
    viable=[t for t in candidates if metrics([r['label'] for r in validation],valid_scores,t)['false_positive_rate']<=.05]
    threshold=min(viable)
    test_scores=[predict(vector(r['text'],vocab,idf),w,b) for r in test]
    model=dict(schema_version=1, status='research_only', tokenizer='ascii_lower_v1', vocabulary=vocab, idf=idf, weights=w, bias=b, threshold=threshold, calibrated=False)
    report=dict(split_seed=42, split_groups=[[r['group'] for r in p] for p in (training,validation,test)], split_sizes=list(map(len,(training,validation,test))), threshold_rule='validation empirical FPR <= 0.05; not a population guarantee', validation=metrics([r['label'] for r in validation],valid_scores,threshold), test=metrics([r['label'] for r in test],test_scores,threshold), limitations=['Research baseline, not approved for production','No independent academic-domain validation','No probability calibration','No domain-shift evaluation','ASCII tokenizer loses non-ASCII information'])
    return model,report


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('dataset'); parser.add_argument('--out',required=True)
    args=parser.parse_args()
    rows=load(args.dataset)
    model,report=train(rows)
    out=Path(args.out)
    out.mkdir(parents=True,exist_ok=False)
    data=json.dumps(model,ensure_ascii=False,sort_keys=True).encode()
    (out/'baseline.json').write_bytes(data)
    report['dataset_sha256']=hashlib.sha256(Path(args.dataset).read_bytes()).hexdigest()
    report['model_sha256']=hashlib.sha256(data).hexdigest()
    report['sources']=sorted({r['source'] for r in rows})
    report['declared_licenses']=sorted({r['license'] for r in rows})
    (out/'evaluation.json').write_text(json.dumps(report,indent=2))
    print(json.dumps(report['test'],indent=2))

if __name__=='__main__': main()
