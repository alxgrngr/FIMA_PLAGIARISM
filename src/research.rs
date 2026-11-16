//! Real trained CPU baseline. Research-only, explicitly opted into by the user.
use leptos::*;
use nusa_ai_detector::{analyze, model::Model};
#[component]
pub fn Research(text: String) -> impl IntoView {
    let (enabled, set_enabled) = create_signal(false);
    view! {<section class="research"><h2>"Model riset Indonesia"</h2>
    <p>"TF-IDF + logistic regression · detectai-topic-v1 · CPU/WASM. Dilatih pada Wikipedia vs Gemma, bukan bukti kepenulisan. Belum diuji independen pada skripsi, ChatGPT, atau teks yang diedit manusia."</p>
    <p>"Test internal: 439 contoh dari topik terpisah; 6/220 contoh berlabel manusia salah ditandai. Skor belum dikalibrasi sebagai probabilitas."</p>
    <label><input type="checkbox" prop:checked=enabled on:change=move |e| set_enabled.set(event_target_checked(&e))/ >" Saya memahami keterbatasan dan ingin menjalankan model riset pada teks ini."</label>
    {move || if enabled.get() { render_scores(&text) } else {view!{<p>"Prediksi model belum dijalankan."</p>}.into_view()}}
    </section>}
}
fn render_scores(text: &str) -> View {
    let model = match Model::from_json(include_bytes!("../models/detectai-topic-v1.json")) {
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
        let result = if !(25..=200).contains(&p.words) {
            Err("Tidak dinilai: perlu 25–200 kata per paragraf (batas riset).")
        } else {
            model.infer(&p.normalized)
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
    view!{<div data-testid="research-result"><h3>"Hasil eksperimen model"</h3><p>{summary}</p><p>"Threshold: 0,4767717384 · dipilih pada validation set dengan FPR empiris ≤5%. Bukan jaminan FPR pada tulisan lain. Paragraf di luar batas panjang dikecualikan dan ditampilkan di bawah."</p>{rows}</div>}.into_view()
}
