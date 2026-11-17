// Cloudflare-specific glue. Document parsing and inference use the Rust WASM core.
import { assess } from './wasm.js';
import QRCode from 'qrcode';
const enc=new TextEncoder();
const json=(data,status=200,headers={})=>new Response(JSON.stringify(data),{status,headers:{'content-type':'application/json; charset=utf-8','cache-control':'no-store','x-content-type-options':'nosniff',...headers}});
const hex=b=>Array.from(new Uint8Array(b),n=>n.toString(16).padStart(2,'0')).join('');
const digest=async s=>hex(await crypto.subtle.digest('SHA-256',typeof s==='string'?enc.encode(s):s));
const random=()=>hex(crypto.getRandomValues(new Uint8Array(32)));
const escape=s=>String(s).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
async function key(env){if(!env.RECEIPT_SIGNING_KEY||env.RECEIPT_SIGNING_KEY.length<64)throw Error('Server signing key missing');return crypto.subtle.importKey('raw',enc.encode(env.RECEIPT_SIGNING_KEY),{name:'HMAC',hash:'SHA-256'},false,['sign','verify']);}
async function sign(payload,env){return hex(await crypto.subtle.sign('HMAC',await key(env),enc.encode(payload)));}
async function valid(payload,sig,env){if(!/^[0-9a-f]{64}$/.test(sig))return false;return crypto.subtle.verify('HMAC',await key(env),Uint8Array.from(sig.match(/../g),b=>parseInt(b,16)),enc.encode(payload));}
async function session(req,env){const token=(req.headers.get('cookie')||'').match(/(?:^|;\s*)__Host-session=([0-9a-f]{64})(?:;|$)/)?.[1];if(!token)return null;return env.DB.prepare('SELECT p.nim,p.name,p.group_name,p.role FROM sessions s JOIN participants p ON p.nim=s.nim WHERE s.token_hash=? AND s.expires>? AND p.active=1').bind(await digest(token),Date.now()).first();}
async function limit(env,name,max,seconds){const now=Date.now();const k=name+':'+Math.floor(now/(seconds*1000));const r=await env.DB.prepare('INSERT INTO limits(key,count,expires) VALUES(?,1,?) ON CONFLICT(key) DO UPDATE SET count=count+1 RETURNING count').bind(k,now+seconds*1000).first();return r.count<=max;}
async function readBounded(req,max){if(Number(req.headers.get('content-length'))>max)throw Error('TOO_LARGE');const reader=req.body?.getReader();if(!reader)return new Uint8Array();const chunks=[];let n=0;while(true){const {done,value}=await reader.read();if(done)break;n+=value.length;if(n>max){await reader.cancel();throw Error('TOO_LARGE');}chunks.push(value);}const result=new Uint8Array(n);let offset=0;for(const c of chunks){result.set(c,offset);offset+=c.length;}return result;}
const cookie=t=>`__Host-session=${t}; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=28800`;
export default {
 async fetch(req,env){try{
 const url=new URL(req.url); const path=url.pathname;
 if(req.method==='POST'&&req.headers.get('origin')!==url.origin)return json({error:'Asal permintaan tidak diizinkan.'},403);
 if(path.startsWith('/api/')&&!env.DB)return json({error:'Database belum dikonfigurasi.'},503);
 if(path==='/api/login'&&req.method==='POST'){
  const ip=await digest(req.headers.get('cf-connecting-ip')||'local');
  if(!await limit(env,'login:'+ip,20,60))return json({error:'Terlalu banyak percobaan. Coba lagi nanti.'},429);
  let data;try{data=JSON.parse(new TextDecoder().decode(await readBounded(req,1024)));}catch{return json({error:'Format login tidak valid.'},400);}
  if(typeof data.nim!=='string'||!/^\d{6,25}$/.test(data.nim))return json({error:'NIM tidak terdaftar.'},401);
  const p=await env.DB.prepare('SELECT nim,name,group_name,role FROM participants WHERE nim=? AND active=1').bind(data.nim).first();
  if(!p)return json({error:'NIM tidak terdaftar.'},401);
  const token=random();await env.DB.prepare('INSERT INTO sessions(token_hash,nim,expires) VALUES(?,?,?)').bind(await digest(token),p.nim,Date.now()+8*3600000).run();
  return json(p,200,{'set-cookie':cookie(token)});
 }
 if(path==='/api/me') {const p=await session(req,env);return p?json(p):json({error:'Silakan masuk.'},401);}
 if(path==='/api/logout'&&req.method==='POST'){
  const token=(req.headers.get('cookie')||'').match(/__Host-session=([0-9a-f]{64})/)?.[1];if(token)await env.DB.prepare('DELETE FROM sessions WHERE token_hash=?').bind(await digest(token)).run();return json({ok:true},200,{'set-cookie':'__Host-session=; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=0'});
 }
 if(path==='/api/receipts'&&req.method==='POST'){
  const p=await session(req,env);if(!p)return json({error:'Silakan masuk.'},401);
  if(req.headers.get('x-consent')!=='temporary-processing')return json({error:'Persetujuan pemrosesan sementara diperlukan.'},400);
  if(!await limit(env,'issue:'+await digest(p.nim),5,3600))return json({error:'Maksimal 5 penerbitan per jam. Coba lagi nanti.'},429);
  const name=url.searchParams.get('name')||'';
  if(name.length<1||name.length>200||/[\x00-\x1f\/\\]/.test(name)||! /\.(txt|docx)$/i.test(name))return json({error:'Nama atau jenis file tidak valid.'},400);
  const bytes=await readBounded(req,10*1024*1024);
  const inferenceStart=performance.now();let analysis;try{analysis=JSON.parse(assess(name,bytes));}catch(e){return json({error:typeof e==='string'?e:'Dokumen tidak dapat dinilai.'},422);}
  const inferenceMs=performance.now()-inferenceStart;const issued=new Date().toISOString();const id=random();
  const payload=JSON.stringify({schema:1,id,issued_at:issued,nim:p.nim,name:p.name,group:p.group_name,kind:p.role==='test'?'TEST':'STUDENT',filename:name,file_sha256:await digest(bytes),...analysis});
  const signature=await sign(payload,env);
  await env.DB.prepare('INSERT INTO receipts(id,nim,payload,signature,created) VALUES(?,?,?,?,?)').bind(id,p.nim,payload,signature,Date.now()).run();
  return json({receipt_url:'/receipt/'+id,verification_url:'/verify/'+id},201,{'server-timing':`inference;dur=${inferenceMs.toFixed(3)}`});
 }
 const match=path.match(/^\/(receipt|verify)\/([0-9a-f]{64})$/);
 if(match&&req.method==='GET'){
  const rec=await env.DB.prepare('SELECT nim,payload,signature FROM receipts WHERE id=?').bind(match[2]).first();if(!rec)return json({error:'Struk tidak ditemukan.'},404);
  if(!await valid(rec.payload,rec.signature,env))return json({error:'Integritas struk tidak valid.'},409);
  const publicView=match[1]==='verify';
  if(!publicView){const p=await session(req,env);if(!p)return json({error:'Silakan masuk.'},401);if(p.nim!==rec.nim)return json({error:'Akses ditolak.'},403);}
  return receipt(JSON.parse(rec.payload),publicView,url.origin);
 }
 if(path.startsWith('/api/')||path.startsWith('/receipt/')||path.startsWith('/verify/'))return json({error:'Tidak ditemukan.'},404);
 return env.ASSETS.fetch(req);
 }catch(e){return json({error:e?.message==='TOO_LARGE'?'File terlalu besar. Maksimal 10 MB.':'Layanan sementara tidak tersedia.'},e?.message==='TOO_LARGE'?413:503);}},
 async scheduled(event,env){await env.DB.batch([env.DB.prepare('DELETE FROM sessions WHERE expires<?').bind(Date.now()),env.DB.prepare('DELETE FROM limits WHERE expires<?').bind(Date.now())]);}
};
async function receipt(r,publicView,origin){
 const time=new Intl.DateTimeFormat('id-ID',{timeZone:'Asia/Jakarta',dateStyle:'full',timeStyle:'long'}).format(new Date(r.issued_at));
 const qr=await QRCode.toString(origin+'/verify/'+r.id,{type:'svg',errorCorrectionLevel:'M',margin:2});
 const person=publicView?r.name.split(/\s+/).map(s=>s[0]+'…').join(' '):r.name;
 const nim=publicView?'••••'+r.nim.slice(-4):r.nim;
 const row=(k,v)=>`<tr><th>${escape(k)}</th><td>${escape(v)}</td></tr>`;
 return new Response(`<!doctype html><html lang="id"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Struk pemeriksaan NUSA</title><style>body{font:15px Arial;color:#17384f;background:#edf3f7;margin:0;padding:24px}main{max-width:660px;margin:auto;padding:28px;background:white;border:1px solid #b9cedd;position:relative;overflow:hidden}h1{font-size:23px}table{width:100%;border-collapse:collapse}th,td{text-align:left;padding:10px 6px;border-bottom:1px solid #dde6ed;vertical-align:top;overflow-wrap:anywhere}th{width:35%}p{line-height:1.6}.stamp{font-size:28px;color:#195887;font-weight:bold}.qr{width:160px}.watermark{position:absolute;inset:0;display:flex;flex-direction:column;justify-content:space-around;pointer-events:none;opacity:.09;font-size:23px;transform:rotate(-25deg);z-index:0}.content{position:relative;z-index:1}.code{overflow-wrap:anywhere;font:12px monospace}.test{color:#9a251e;font-weight:bold;border:2px solid;padding:10px}a{color:#195887}small{font-size:12px}@media print{body{padding:0;background:white}main{border:0}}</style></head><body><main><div class="watermark" aria-hidden="true">${Array(7).fill(`<span>${escape(time)}</span>`).join('')}</div><div class="content"><h1>${publicView?'Verifikasi catatan struk':'Struk pemeriksaan laporan'}</h1><p>Perilaku Organisasi · Teknik Industri UMM<br>Alat bantu eksperimen, bukan layanan resmi universitas</p>${r.kind==='TEST'?'<p class="test">UJI ADMIN — BUKAN BUKTI PENGUMPULAN</p>':''}<p class="stamp">${r.percentage.toFixed(1)}% kata pada paragraf ditandai</p><p>Bukan probabilitas kepenulisan AI. Model riset dapat salah; bukan bukti pelanggaran akademik.</p><table>${row('Nama',person)+row('NIM',nim)+row('Kelompok',r.group)+row('Waktu server (WIB)',time)+row('Nama file',r.filename)+row('Kata dinilai',r.eligible_words)+row('Kata dikecualikan',r.excluded_words)+row('Model',r.model)+row('Threshold',r.threshold)}</table><p class="code">ID: ${escape(r.id)}<br>SHA-256 file: ${escape(r.file_sha256)}</p><div class="qr">${qr}</div><p><a href="/verify/${r.id}">Buka catatan asli untuk verifikasi</a></p><p><strong>Screenshot struk ini dan kirimkan kepada aslab. Pastikan ID pemeriksaan dan QR code terlihat.</strong></p><small>${publicView?'Tanda tangan server valid. Data identitas disamarkan pada tautan publik.':'Hanya penerbitan server yang tercatat. Mengedit screenshot tidak mengubah catatan verifikasi.'} Cocokkan hash dengan file yang dikumpulkan. Login NIM tidak membuktikan identitas pemilik NIM. Dokumen tidak disimpan; catatan hasil disimpan untuk verifikasi.</small></div></main></body></html>`,{headers:{'content-type':'text/html; charset=utf-8','cache-control':'no-store','content-security-policy':"default-src 'none'; style-src 'unsafe-inline'; img-src 'self' data:; base-uri 'none'; frame-ancestors 'none'; form-action 'none'",'referrer-policy':'no-referrer','x-content-type-options':'nosniff'}});
}
