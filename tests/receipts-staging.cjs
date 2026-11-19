// Remote staging only. Uses randomly generated synthetic accounts from private state.
const fs=require('fs');const assert=require('assert/strict');const crypto=require('crypto');
const {execFileSync}=require('node:child_process');
const base=process.env.NUSA_TEST_BASE_URL||'https://nusa-receipts-staging.fiqihh413.workers.dev';
if(new URL(base).hostname!=='nusa-receipts-staging.fiqihh413.workers.dev')throw Error('This runner is restricted to staging');
const users=JSON.parse(fs.readFileSync('private/staging-users.json','utf8'));
const profile=process.env.NUSA_MODEL_PROFILE||'journal-pilot-v1';
const modelFiles={
 'detectai-topic-v1':'models/detectai-topic-v1.json',
 'journal-pilot-v1':process.env.NUSA_JOURNAL_MODEL_PATH||'training/runs/journal-three-pilot-2026-11-19/journal-pilot.json',
};
function referenceDecision(text) {
 if (!Object.hasOwn(modelFiles,profile)) throw Error('Unknown staging model profile');
 const script = [
  'import json, sys',
  'from pathlib import Path',
  'sys.path.insert(0, "training")',
  'from baseline import predict, vector',
  'model = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))',
  'features = vector(sys.stdin.buffer.read().decode("utf-8"), model["vocabulary"], model["idf"])',
  'assert features',
  'print(int(predict(features, model["weights"], model["bias"]) >= model["threshold"]))',
 ].join('\n');
 return execFileSync('python3',['-c',script,modelFiles[profile]],{input:text,encoding:'utf8'}).trim()==='1';
}
async function req(path,options={}){return fetch(base+path,{...options,headers:{origin:base,...options.headers}});}
async function login(nim){const r=await req('/api/login',{method:'POST',body:JSON.stringify({nim})});assert.equal(r.status,200,'Staging login failed');return r.headers.get('set-cookie').split(';')[0];}
(async()=>{
 assert.equal((await req('/api/me')).status,401);
 const owner=await login(users[0].nim), other=await login(users[1].nim), tester=await login(users[2].nim);
 const c=JSON.parse(fs.readFileSync('tests/model-cases.json'))[0];const bytes=Buffer.from(c.text);
 assert.equal((await req('/api/receipts?name=example.txt',{method:'POST',body:bytes})).status,401);
 assert.equal((await req('/api/receipts?name=example.txt',{method:'POST',body:bytes,headers:{cookie:owner}})).status,400);
 const r=await req('/api/receipts?name=example.txt',{method:'POST',body:bytes,headers:{cookie:owner,'x-consent':'temporary-processing','x-score':'99','x-issued-at':'2000-01-01'}});
 assert.equal(r.status,201,'Staging server could not issue receipt');const links=await r.json();
 const html=await(await req(links.receipt_url,{headers:{cookie:owner}})).text();assert.ok(html.includes(crypto.createHash('sha256').update(bytes).digest('hex')));assert.ok(!html.includes('2000-01-01'));assert.ok(html.includes('<svg'));
 assert.ok(html.includes(profile), 'Staging receipt uses the wrong model');
 assert.ok(html.includes((referenceDecision(c.text)?'100.0':'0.0')+'% kata'));
 assert.equal((await req(links.receipt_url,{headers:{cookie:other}})).status,403);
 const publicResult=await req(links.verification_url);assert.equal(publicResult.status,200);const publicHtml=await publicResult.text();assert.ok(!publicHtml.includes(users[0].nim));
 const testResult=await req('/api/receipts?name=example.txt',{method:'POST',body:bytes,headers:{cookie:tester,'x-consent':'temporary-processing'}});assert.equal(testResult.status,201);const t=await testResult.json();assert.ok((await(await req(t.receipt_url,{headers:{cookie:tester}})).text()).includes('UJI ADMIN'));
 await req('/api/logout',{method:'POST',headers:{cookie:owner}});assert.equal((await req('/api/me',{headers:{cookie:owner}})).status,401);
 console.log('PASS: remote staging login, server scoring, consent, hash, QR, ownership, public masking, test-role mark and logout. This alone is not a CPU-capacity benchmark.');
})().catch(e=>{console.error(e.message);process.exit(1)});
