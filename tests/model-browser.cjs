const { chromium } = require('playwright');
const fs = require('fs');
(async()=>{
 const cases=JSON.parse(fs.readFileSync(process.argv[2] || 'tests/model-cases.json','utf8'));
 const browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH||'/usr/bin/chromium',headless:true,args:['--no-sandbox','--disable-features=WebGPU']});
 const page=await browser.newPage();const errors=[]; const outbound=[];
 page.on('pageerror',e=>errors.push(e.message));
 page.on('request',r=>{if(new URL(r.url()).hostname!=='127.0.0.1')outbound.push(r.url());});
 await page.goto('http://127.0.0.1:8081');
 for(const c of cases){
  await page.locator('#text').fill(c.text);
  await page.getByRole('button',{name:'Periksa laporan'}).click();
  await page.getByText('Prediksi model belum dijalankan.').waitFor();
  const begin=Date.now();
  await page.locator('.research input[type=checkbox]').check();
  const result=page.getByTestId('research-result');await result.waitFor();
  const content=await result.innerText();
  if(!content.includes(c.score.toFixed(4)))throw Error('Python/WASM score mismatch');
  if(!content.includes('BUKAN probabilitas'))throw Error('Missing interpretation');
  console.log('PASS: real-model WASM score parity; render latency ms',Date.now()-begin);
 }
 const threshold=JSON.parse(fs.readFileSync('models/model-manifest.json','utf8')).threshold;
 const total=cases.reduce((n,c)=>n+c.text.trim().split(/\s+/).length,0);
 const flagged=cases.filter(c=>c.score>=threshold).reduce((n,c)=>n+c.text.trim().split(/\s+/).length,0);
 await page.locator('#text').fill(cases.map(c=>c.text).join('\n\n')+'\n\nTeks pendek.');
 await page.getByRole('button',{name:'Periksa laporan'}).click();
 await page.locator('.research input[type=checkbox]').check();
 const combined=await page.getByTestId('research-result').innerText();
 if(!combined.includes((flagged/total*100).toFixed(1)+'%')||!combined.includes(flagged+' / '+total+' kata'))throw Error('Word-weighted aggregation mismatch');
 if(!combined.includes('Dilewati: kurang dari 25 kata.'))throw Error('Missing excluded paragraph');
 console.log('PASS: word-weighted aggregation and short-paragraph exclusion');
 await page.locator('#text').fill('Teks pendek.');await page.getByRole('button',{name:'Periksa laporan'}).click();await page.locator('.research input[type=checkbox]').check();await page.getByText('Tidak ada paragraf memenuhi syarat. Tidak ada persentase AI yang dihitung.').waitFor();
 if(errors.length||outbound.length)throw Error(JSON.stringify({errors,outbound}));
 console.log('PASS: short-text abstention; CPU execution with WebGPU disabled; no external requests or page errors');
 await browser.close();
})().catch(e=>{console.error(e);process.exit(1)});
