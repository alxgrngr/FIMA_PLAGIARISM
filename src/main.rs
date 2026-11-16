use leptos::*;
use nusa_ai_detector::analyze;
use wasm_bindgen::JsCast;
fn main() {
    mount_to_body(|| view! { <App/> });
}
#[component]
fn App() -> impl IntoView {
    let (text, set_text) = create_signal(String::new());
    let (result, set_result) = create_signal(None);
    let generation = create_rw_signal(0_u64);
    let (file_status, set_file_status) = create_signal(String::new());
    view! {
            <header><a href="#" class="brand">"NUSA"<span>"AI DETECTOR"</span></a><nav><a href="#workspace">"Analisis"</a><a href="#about">"Tentang & privasi"</a></nav></header>
            <main>
            <section class="hero"><span class="eyebrow">"BAHASA INDONESIA · PRIVASI DI PERANGKAT"</span><h1>"Kenali teks Anda."<br/><em>"Pahami batas analisis AI."</em></h1><p>"Ruang analisis dokumen yang transparan. Teks diproses di browser Anda, tanpa akun dan tanpa mengunggah dokumen."</p><div class="badge">"Prototipe riset · Model belum tersedia"</div></section>
            <section id="workspace" class="workspace"><div class="editor"><div class="section-title"><h2>"Mulai dari teks"</h2><button class="quiet" on:click=move |_| {generation.update(|g| *g += 1); set_file_status.set(String::new()); set_text.set(String::new()); set_result.set(None);}>"Hapus"</button></div><label for="file">"Buka TXT UTF-8 (maks. 400 KB)"</label><input id="file" type="file" accept=".txt,text/plain" on:change=move |ev| {
    generation.update(|g| *g += 1);
    let current = generation.get_untracked();
    let input = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    if let Some(file) = input.files().and_then(|f| f.get(0)) {
    if file.size() > 400_000.0 { set_file_status.set("File terlalu besar.".into()); return; }
    if !file.name().to_lowercase().ends_with(".txt") {set_file_status.set("Hanya TXT didukung saat ini.".into()); return;}
    set_file_status.set("Membaca file…".into());
    spawn_local(async move { let loaded = wasm_bindgen_futures::JsFuture::from(file.text()).await;
    if current != generation.get_untracked() {return;}
    match loaded {
    Ok(value) => {let value=value.as_string().unwrap_or_default(); if value.contains('\0') || value.contains('\u{fffd}') {set_file_status.set("File bukan teks UTF-8 yang valid.".into());} else if value.chars().count()>100_000 {set_file_status.set("Teks melebihi 100.000 karakter.".into());} else {set_text.set(value);set_result.set(None);set_file_status.set("Teks dimuat. Tinjau sebelum pemeriksaan.".into());}},
    Err(_) => set_file_status.set("File gagal dibaca.".into())
    }});
    }}/><p role="status">{file_status}</p><label for="text">"Tempel teks berbahasa Indonesia"</label><textarea id="text" placeholder="Tempel artikel, laporan, atau paragraf Anda di sini…" prop:value=text on:input=move |ev| {generation.update(|g| *g += 1); set_text.set(event_target_value(&ev)); set_result.set(None);}></textarea><div class="counter">{move || format!("{} kata · {} / 100.000 karakter", text.get().split_whitespace().count(),text.get().chars().count())}</div><button class="primary" on:click=move |_| set_result.set(Some(analyze(&text.get())))>"Periksa teks lokal"</button><p class="note">"Pemeriksaan ini menampilkan statistik, bukan prediksi AI. DOCX dan PDF belum tersedia."</p></div>
            <aside><span class="eyebrow">"STATUS MODEL"</span><h2>"Kejujuran sebelum angka."</h2><p>"Belum ada model deteksi berbahasa Indonesia yang terintegrasi dan tervalidasi. Persentase AI tidak ditampilkan."</p><ul><li>"Tanpa API AI berbayar"</li><li>"Tanpa penyimpanan teks otomatis"</li><li>"Statistik tidak membuktikan asal tulisan"</li></ul></aside></section>
            <section aria-live="polite">{move || result.get().map(|r| match r {Err(e)=>view!{<div class="alert">{e}</div>}.into_view(),Ok(a)=>view!{<div class="results"><h2>"Ringkasan teks"</h2><div class="metrics"><strong>{format!("{} kata",a.words)}</strong><strong>{format!("{} paragraf",a.paragraphs.len())}</strong><strong>{format!("{:.0}% keragaman leksikal",a.lexical_diversity*100.0)}</strong></div><p>"Model belum tersedia · Tidak ada segmen yang diklasifikasikan. Semua paragraf disertakan; tidak ada bagian yang dihapus."</p><p>{if a.words<100 {"Teks pendek: konteks terbatas. Bahasa belum diverifikasi otomatis."}else{"Bahasa belum diverifikasi otomatis."}}</p>{a.paragraphs.into_iter().enumerate().map(|(i,p)|view!{<article><small>{format!("Paragraf {} · {} kata · belum dianalisis model",i+1,p.words)}</small><p>{p.normalized}</p></article>}).collect_view()}</div>}.into_view()})}</section>
            <section id="about" class="about"><h2>"Analisis yang bertanggung jawab"</h2><details><summary>"Apakah hasil membuktikan penggunaan AI?"</summary><p>"Tidak. Bahkan model tervalidasi dapat salah. Jangan gunakan detektor sebagai satu-satunya dasar keputusan akademik."</p></details><details><summary>"Ke mana teks saya dikirim?"</summary><p>"Teks tetap di memori browser. Aplikasi tidak mengirim teks ke server dan tidak menyimpannya secara persisten. Hapus teks atau tutup halaman untuk mengakhiri sesi."</p></details></section>
            </main><footer>"NUSA AI Detector · Prototipe riset · Bukan bukti kepenulisan"</footer>
        }
}
