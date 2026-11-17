const assert=require('node:assert/strict');const fs=require('fs');const crypto=require('crypto');
const base='http://127.0.0.1:8787';
async function call(path,{method='GET',cookie='',body,headers={}}={}){return fetch(base+path,{method,headers:{origin:base,...(cookie?{cookie}:{}),...headers},body});}
async function login(nim){const r=await call('/api/login',{method:'POST',body:JSON.stringify({nim}),headers:{'content-type':'application/json'}});assert.equal(r.status,200);const c=r.headers.get('set-cookie');assert.match(c,/HttpOnly/);assert.match(c,/Secure/);assert.match(c,/SameSite=Strict/);return c.split(';')[0];}
(async()=>{
 assert.equal((await call('/api/me')).status,401);
 assert.equal((await call('/api/login',{method:'POST',body:JSON.stringify({nim:'111111'})})).status,401);
 assert.equal((await call('/api/login',{method:'POST',body:'{}',headers:{origin:'https://evil.example'}})).status,403);
 const owner=await login('999001');const other=await login('999002');const test=await login('999003');
 const cases=JSON.parse(fs.readFileSync('tests/model-cases.json'));const bytes=Buffer.from(cases[0].text);const filename='<img onerror=alert(1)>.txt';
 assert.equal((await call('/api/receipts?name=a.txt',{method:'POST',body:bytes})).status,401);
 assert.equal((await call('/api/receipts?name=a.txt',{method:'POST',cookie:owner,body:bytes})).status,400);
 const r=await call('/api/receipts?name='+encodeURIComponent(filename),{method:'POST',cookie:owner,body:bytes,headers:{'x-consent':'temporary-processing','x-score':'99','x-issued-at':'2000-01-01'}});
 assert.equal(r.status,201);console.log('Local synchronous inference wall timing (not Cloudflare CPU):',r.headers.get('server-timing'));
 const links=await r.json();const receipt=await call(links.receipt_url,{cookie:owner});assert.equal(receipt.status,200);const html=await receipt.text();
 const threshold=JSON.parse(fs.readFileSync('models/model-manifest.json')).threshold;assert.ok(html.includes((cases[0].score>=threshold?'100.0':'0.0')+'% kata'));assert.ok(html.includes(crypto.createHash('sha256').update(bytes).digest('hex')));assert.ok(html.includes('&lt;img'));assert.ok(!html.includes('<img onerror'));assert.ok(!html.includes('2000-01-01'));assert.ok(html.includes('watermark'));assert.ok(html.includes('<svg'));assert.ok(html.includes('Peserta Uji A'));
 assert.equal((await call(links.receipt_url,{cookie:other})).status,403);assert.equal((await call(links.receipt_url)).status,401);
 const verify=await call(links.verification_url);assert.equal(verify.status,200);const vh=await verify.text();assert.ok(!vh.includes('Peserta Uji A'));assert.ok(!vh.includes('999001'));assert.ok(vh.includes('Tanda tangan server valid'));
 const tr=await call('/api/receipts?name=test.txt',{method:'POST',cookie:test,body:bytes,headers:{'x-consent':'temporary-processing'}});assert.equal(tr.status,201);const tl=await tr.json();assert.ok((await (await call(tl.receipt_url,{cookie:test})).text()).includes('UJI ADMIN — BUKAN BUKTI PENGUMPULAN'));
 const malformed=await call('/api/receipts?name=fake.docx',{method:'POST',cookie:owner,body:'not zip',headers:{'x-consent':'temporary-processing'}});assert.equal(malformed.status,422);
 fs.writeFileSync('/tmp/nusa-receipt-test-id',links.receipt_url.split('/').pop());
 await call('/api/logout',{method:'POST',cookie:owner});assert.equal((await call('/api/me',{cookie:owner})).status,401);
 console.log('PASS: allowlist login, cookie flags, CSRF, auth/ownership, consent, server inference ignores client score/time, file hash, escaped HTML, public masking, QR, test marking, corrupt DOCX, logout');
})().catch(e=>{console.error(e);process.exit(1)});
