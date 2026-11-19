//! Real trained CPU baseline. Research-only, explicitly opted into by the user.
use leptos::*;
use nusa_ai_detector::{
    analyze,
    model::{Model, BUNDLED_MODEL_ID},
    paragraph_exclusion,
};
#[component]
pub fn Research(text: String) -> impl IntoView {
    let (enabled, set_enabled) = create_signal(false);
    view! {<section class="research"><h2>"Pemeriksaan indikasi AI"</h2>
    <p>"Model eksperimental ini belum divalidasi khusus untuk laporan praktikum Perilaku Organisasi. Hasil dapat salah dan perlu ditinjau bersama isi laporan."</p>
    <details><summary>"Model dan hasil pengujian"</summary>
    <p>{BUNDLED_MODEL_ID}" · TF-IDF dan regresi logistik."</p>
    <p>{if BUNDLED_MODEL_ID == "journal-pilot-v1" {
        "Uji dengan menahan satu artikel bergantian: model menandai 9 dari 10 paragraf AI dan 1 dari 25 paragraf jurnal. Data berasal dari tiga jurnal dan contoh buatan Qwen serta Llama."
    } else {
        "Data latihan: Wikipedia dan Gemma. Tes internal: 439 contoh dari topik terpisah; 6 dari 220 contoh berlabel manusia ditandai."
    }}</p></details>
    <label><input type="checkbox" prop:checked=enabled on:change=move |e| set_enabled.set(event_target_checked(&e))/ >" Saya memahami keterbatasan dan ingin menjalankan model riset pada teks ini."</label>
    {move || if enabled.get() { render_scores(&text) } else {view!{<p>"Prediksi model belum dijalankan."</p>}.into_view()}}
    </section>}
}
fn render_scores(text: &str) -> View {
    let model = match Model::bundled() {
        Ok(m) => m,
        Err(e) => return view! {<p role="alert">{e}</p>}.into_view(),
    };
    let analysis = match analyze(text) {
        Ok(a) => a,
        Err(e) => return view! {<p role="alert">{e}</p>}.into_view(),
    };
    let mut eligible = 0;
    let mut flagged = 0;
    let mut rows = Vec::new();
    for (i, p) in analysis.paragraphs.iter().enumerate() {
        let result = match paragraph_exclusion(p.words) {
            Some(reason) => Err(reason),
            None => model.infer(&p.normalized),
        };
        let (label, class) = match result {
            Ok(s) => {
                eligible += p.words;
                if s.flagged {
                    flagged += p.words;
                }
                (
                    format!(
                        "{} · skor model {:.4} · cakupan kosakata {:.0}%",
                        if s.flagged {
                            "Ditandai pada threshold riset"
                        } else {
                            "Tidak ditandai; bukan berarti pasti manusia"
                        },
                        s.score,
                        s.vocabulary_coverage * 100.0
                    ),
                    if s.flagged { "flagged" } else { "unflagged" },
                )
            }
            Err(e) => (e.to_string(), "uncertain"),
        };
        rows.push(view!{<article class=class><strong>{format!("Paragraf {} · {} kata",i+1,p.words)}</strong><p>{label}</p><p>{p.normalized.clone()}</p></article>});
    }
    let summary = if eligible == 0 {
        "Tidak ada paragraf memenuhi syarat. Tidak ada persentase AI yang dihitung.".to_string()
    } else {
        format!("{:.1}% kata dalam segmen yang dinilai berada pada segmen ditandai ({} / {} kata). Ini BUKAN probabilitas seluruh dokumen ditulis AI.",flagged as f64/eligible as f64*100.0,flagged,eligible)
    };
    let threshold = format!("{:.10}", model.threshold())
        .trim_end_matches('0')
        .trim_end_matches('.')
        .replace('.', ",");
    view!{<div data-testid="research-result"><h3>"Hasil pemeriksaan"</h3><p>{summary}</p>
        <p>{format!("Ambang penandaan model: {threshold}. Hasil bisa keliru. Paragraf di luar batas panjang ditampilkan sebagai dilewati.")}</p>{rows}</div>}.into_view()
}
