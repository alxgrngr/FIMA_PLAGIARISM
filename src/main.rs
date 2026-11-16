mod research;
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
                <header><a href="#" class="brand">"NUSA / AI Detector"</a><a href="#about">"Tentang alat"</a></header>
                <main>
                <section class="intro"><p class="course">"Teknik Industri · Universitas Muhammadiyah Malang"</p><h1>"Pemeriksaan laporan praktikum"</h1><p class="subject">"Perilaku Organisasi"</p><p class="description">"Tempel isi laporan untuk memeriksa indikasi teks AI per paragraf. Teks diproses di browser dan tidak diunggah."</p></section>
                <section id="workspace" class="workspace"><div class="editor"><div class="section-title"><h2>"Isi laporan"</h2><button class="quiet" on:click=move |_| {generation.update(|g| *g += 1); set_file_status.set(String::new()); set_text.set(String::new()); set_result.set(None);}>"Hapus"</button></div><label for="file">"Buka TXT UTF-8 (maks. 400 KB)"</label><input id="file" type="file" accept=".txt,text/plain" on:change=move |ev| {
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
        }}/><p role="status">{file_status}</p><label for="text">"Teks laporan"</label><textarea id="text" placeholder="Tempel bagian laporan yang ingin diperiksa. Pisahkan setiap paragraf dengan baris baru." prop:value=text on:input=move |ev| {generation.update(|g| *g += 1); set_text.set(event_target_value(&ev)); set_result.set(None);}></textarea><div class="counter">{move || format!("{} kata · {} / 100.000 karakter", text.get().split_whitespace().count(),text.get().chars().count())}</div><button class="primary" on:click=move |_| set_result.set(Some(analyze(&text.get())))>"Periksa laporan"</button><p class="note">"Gunakan paragraf 25–200 kata. Setelah pemeriksaan, aktifkan model riset untuk melihat indikasi AI. Format file yang didukung: TXT; belum DOCX/PDF."</p></div>
    </section>
                <section aria-live="polite">{move || result.get().map(|r| match r {Err(e)=>view!{<div class="alert">{e}</div>}.into_view(),Ok(a)=>view!{<div class="results"><h2>"Ringkasan laporan"</h2><div class="metrics"><strong>{format!("{} kata",a.words)}</strong><strong>{format!("{} paragraf",a.paragraphs.len())}</strong><strong>{format!("{:.0}% keragaman leksikal",a.lexical_diversity*100.0)}</strong></div><p class="note">"Jumlah kata dan keragaman leksikal adalah statistik, bukan hasil deteksi AI."</p><p>{if a.words<100 {"Teks pendek: konteks terbatas. Bahasa belum diverifikasi otomatis."}else{"Bahasa belum diverifikasi otomatis."}}</p><details><summary>"Lihat teks per paragraf"</summary>{a.paragraphs.into_iter().enumerate().map(|(i,p)|view!{<article><small>{format!("Paragraf {} · {} kata · belum dianalisis model",i+1,p.words)}</small><p>{p.normalized}</p></article>}).collect_view()}</details></div>}.into_view()})}</section>
                {move || result.get().and_then(|r| r.ok()).map(|_| view!{<research::Research text=text.get()/>})}
                <section id="about" class="about"><h2>"Catatan penggunaan"</h2><details><summary>"Apakah hasil membuktikan penggunaan AI?"</summary><p>"Tidak. Model ini belum divalidasi khusus pada laporan praktikum Perilaku Organisasi Teknik Industri UMM. Hasilnya hanya bahan peninjauan, bukan bukti pelanggaran atau dasar tunggal penilaian."</p></details><details><summary>"Ke mana teks saya dikirim?"</summary><p>"Teks tetap di memori browser. Aplikasi tidak mengirim teks ke server dan tidak menyimpannya secara persisten. Hapus teks atau tutup halaman untuk mengakhiri sesi."</p></details></section>
                </main><footer>"NUSA · Alat bantu peninjauan laporan · Bukan layanan resmi Universitas Muhammadiyah Malang"</footer>
            }
}
