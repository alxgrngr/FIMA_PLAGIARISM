"""Additional locked-test diagnostics; does not retune model or threshold."""
import json,math,csv,sys
from pathlib import Path
from baseline import load,split,vector,predict,tokens,metrics
m=json.loads(Path('models/detectai-topic-v1.json').read_text())
train,valid,test=split(load('datasets/local/prepared.jsonl'))
scores=[predict(vector(r['text'],m['vocabulary'],m['idf']),m['weights'],m['bias']) for r in test]
y=[r['label'] for r in test]
pos=[p for p,l in zip(scores,y) if l];neg=[p for p,l in zip(scores,y) if not l]
auc=sum((a>b)+.5*(a==b) for a in pos for b in neg)/(len(pos)*len(neg))
# Stepwise area under precision-recall curve / average precision, tie-aware.
ap=0; previous_recall=0
for t in sorted(set(scores),reverse=True):
 tp=sum(l==1 and p>=t for l,p in zip(y,scores)); fp=sum(l==0 and p>=t for l,p in zip(y,scores))
 recall=tp/len(pos);ap+=(recall-previous_recall)*tp/(tp+fp);previous_recall=recall
ece=0
for i in range(10):
 pairs=[(p,l) for p,l in zip(scores,y) if i/10<=p<(i+1)/10 or i==9 and p==1]
 if pairs:ece+=len(pairs)/len(y)*abs(sum(p for p,l in pairs)/len(pairs)-sum(l for p,l in pairs)/len(pairs))
train_sets=[set(tokens(r['text'])) for r in train+valid]
near=sum(any(len(s&t)/max(1,len(s|t))>=.8 for t in train_sets) for s in [set(tokens(r['text'])) for r in test])
raw={r['text']:r for r in csv.DictReader(open('datasets/local/detectai.csv',encoding='utf-8-sig'))}
by_model={}
for name in sorted({raw[r['text']]['model'] or 'publisher-labelled-human' for r in test}):
 indices=[i for i,r in enumerate(test) if (raw[r['text']]['model'] or 'publisher-labelled-human')==name]
 by_model[name]=metrics([y[i] for i in indices],[scores[i] for i in indices],m['threshold'])
report=dict(roc_auc=auc,average_precision=ap,ece_10_bins=ece,test_near_overlap_with_train_validation_jaccard_08=near,by_generator=by_model,notes=['Diagnostic only; no test threshold tuning','Token-set Jaccard is not a semantic leakage audit','Wikipedia source and human class are confounded; AI class uses Gemma only','No independent academic test corpus'])
Path('models/diagnostics-v1.json').write_text(json.dumps(report,indent=2))
print(json.dumps(report,indent=2))
