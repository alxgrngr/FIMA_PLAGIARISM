#[cfg(feature = "verified-receipts")]
mod auth;
mod research;
use leptos::*;
use nusa_ai_detector::analyze;
use wasm_bindgen::JsCast;
fn main() {
    #[cfg(feature = "verified-receipts")]
    mount_to_body(|| view! { <auth::LoginGate/> });
    #[cfg(not(feature = "verified-receipts"))]
    mount_to_body(|| view! { <App/> });
}
#[component]
fn App() -> impl IntoView {
    let upload = store_value::<Option<(String, Vec<u8>)>>(None);
    let (text, set_text) = create_signal(String::new());
    let (result, set_result) = create_signal(None);
    let generation = create_rw_signal(0_u64);
    let (file_status, set_file_status) = create_signal(String::new());
    view! {
                <header><a href="#" class="brand">"NUSA / INDUSTRIAL ENGINEERING"</a><a href="#about">"Tentang alat"</a></header>
                <main>
                <section class="intro"><p class="course">"Teknik Industri · Universitas Muhammadiyah Malang"</p><h1>"Pemeriksaan laporan praktikum"</h1><p class="subject">"Perilaku Organisasi & Manajemen Perusahaan"</p><p class="description">"Tinjau laporan tentang kepemimpinan, kerja tim, dan sistem organisasi. Unggah DOCX atau tempel teks; pratinjau dan pemeriksaan awal dilakukan di perangkat Anda."</p><div class="process" aria-label="Alur pemeriksaan"><span><b>01</b> Dokumen laporan</span><span><b>02</b> Tinjauan paragraf</span><span><b>03</b> Evaluasi hasil</span></div></section>
                <section id="workspace" class="workspace"><div class="editor"><div class="section-title"><h2>"Isi laporan"</h2><button class="quiet" on:click=move |_| {upload.set_value(None); generation.update(|g| *g += 1); set_file_status.set(String::new()); set_text.set(String::new()); set_result.set(None);}>"Hapus"</button></div><label for="file">"Upload laporan · DOCX / TXT · maks. 10 MB"</label><input id="file" type="file" accept=".docx,.txt,application/vnd.openxmlformats-officedocument.wordprocessingml.document,text/plain" on:change=move |ev| {
        generation.update(|g| *g += 1);
        upload.set_value(None);
        let current = generation.get_untracked();
        let input = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        if let Some(file) = input.files().and_then(|f| f.get(0)) {
        if file.size() > nusa_ai_detector::documents::MAX_FILE_BYTES as f64 { set_file_status.set("File terlalu besar. Maksimal 10 MB (10.485.760 byte).".into()); return; }
        let name=file.name();
        if !name.to_lowercase().ends_with(".docx") && !name.to_lowercase().ends_with(".txt") {set_file_status.set("Gunakan file DOCX atau TXT. PDF dan DOC belum didukung.".into());return;}
        set_file_status.set("Membaca dan mengekstrak dokumen…".into());
        spawn_local(async move {
            let loaded=wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await;
            if current != generation.get_untracked() {return;}
            match loaded {
                Ok(value)=>{let bytes=js_sys::Uint8Array::new(&value).to_vec(); match nusa_ai_detector::documents::extract(&name,&bytes) {
                    Ok(value)=>{upload.set_value(Some((name.clone(),bytes)));set_text.set(value);set_result.set(None);set_file_status.set("Teks dimuat. Tinjau sebelum pemeriksaan.".into());},
                    Err(e)=>set_file_status.set(e.into())
                }},
                Err(_)=>set_file_status.set("File gagal dibaca.".into())
            }
        });
        }}/><p role="status">{file_status}</p><label for="text">"Teks laporan"</label><textarea id="text" placeholder="Tempel bagian laporan yang ingin diperiksa. Pisahkan setiap paragraf dengan baris baru." prop:value=text on:input=move |ev| {upload.set_value(None); generation.update(|g| *g += 1); set_text.set(event_target_value(&ev)); set_result.set(None);}></textarea><div class="counter">{move || format!("{} kata · {} / 100.000 karakter", text.get().split_whitespace().count(),text.get().chars().count())}</div><button class="primary" on:click=move |_| set_result.set(Some(analyze(&text.get())))>"Periksa laporan"</button><p class="note">"Gunakan paragraf 25–200 kata. Setelah pemeriksaan, aktifkan model riset untuk melihat indikasi AI. DOCX: teks utama dan isi tabel dibaca; gambar, header/footer, catatan kaki, dan format visual tidak disertakan. Maksimal 100.000 karakter teks."</p></div>
    </section>
                <section aria-live="polite">{move || result.get().map(|r| match r {Err(e)=>view!{<div class="alert">{e}</div>}.into_view(),Ok(a)=>view!{<div class="results"><h2>"Ringkasan laporan"</h2><div class="metrics"><strong>{format!("{} kata",a.words)}</strong><strong>{format!("{} paragraf",a.paragraphs.len())}</strong><strong>{format!("{:.0}% keragaman leksikal",a.lexical_diversity*100.0)}</strong></div><p class="note">"Jumlah kata dan keragaman leksikal adalah statistik, bukan hasil deteksi AI."</p><p>{if a.words<100 {"Teks pendek: konteks terbatas. Bahasa belum diverifikasi otomatis."}else{"Bahasa belum diverifikasi otomatis."}}</p><details><summary>"Lihat teks per paragraf"</summary>{a.paragraphs.into_iter().enumerate().map(|(i,p)|view!{<article><small>{format!("Paragraf {} · {} kata · belum dianalisis model",i+1,p.words)}</small><p>{p.normalized}</p></article>}).collect_view()}</details></div>}.into_view()})}</section>
                {move || result.get().and_then(|r| r.ok()).map(|_| view!{<research::Research text=text.get()/>})}
                {receipt_controls(upload)}
                <section id="about" class="about"><h2>"Catatan penggunaan"</h2><details><summary>"Apakah hasil membuktikan penggunaan AI?"</summary><p>"Tidak. Model ini belum divalidasi khusus pada laporan praktikum Perilaku Organisasi Teknik Industri UMM. Hasilnya hanya bahan peninjauan, bukan bukti pelanggaran atau dasar tunggal penilaian."</p></details><details><summary>"Ke mana teks saya dikirim?"</summary><p>"Pratinjau dan analisis awal berjalan lokal. Pada mode struk terverifikasi, file dikirim sementara ke server setelah persetujuan Anda. Isi file tidak disimpan; identitas dan catatan hasil disimpan untuk verifikasi aslab."</p></details></section>
                </main><footer>"NUSA · Alat bantu peninjauan laporan · Bukan layanan resmi Universitas Muhammadiyah Malang"</footer>
            }
}

fn receipt_controls(upload: StoredValue<Option<(String, Vec<u8>)>>) -> View {
    #[cfg(feature = "verified-receipts")]
    {
        view! { <auth::ReceiptControls upload=upload/> }.into_view()
    }
    #[cfg(not(feature = "verified-receipts"))]
    {
        let _ = upload;
        ().into_view()
    }
}
