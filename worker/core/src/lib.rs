use nusa_ai_detector::{analyze, documents, model::Model};
use std::sync::OnceLock;
use wasm_bindgen::prelude::*;
static MODEL: OnceLock<Result<Model, &'static str>> = OnceLock::new();
fn model() -> Result<&'static Model, JsValue> {
    MODEL
        .get_or_init(|| Model::from_json(include_bytes!("../../../models/detectai-topic-v1.json")))
        .as_ref()
        .map_err(|e| JsValue::from_str(e))
}
#[wasm_bindgen]
pub fn warm_model() -> Result<(), JsValue> {
    model().map(|_| ())
}
#[wasm_bindgen]
pub fn assess(name: &str, bytes: &[u8]) -> Result<String, JsValue> {
    let text = documents::extract(name, bytes).map_err(JsValue::from_str)?;
    let doc = analyze(&text).map_err(JsValue::from_str)?;
    let model = model()?;
    let mut eligible = 0usize;
    let mut flagged = 0usize;
    let mut segments = Vec::new();
    for (i, p) in doc.paragraphs.iter().enumerate() {
        if !(25..=200).contains(&p.words) {
            segments.push(
                serde_json::json!({"paragraph":i+1,"words":p.words,"status":"excluded_length"}),
            );
            continue;
        }
        match model.infer(&p.normalized) {
            Ok(s) => {
                eligible += p.words;
                if s.flagged {
                    flagged += p.words;
                }
                segments.push(serde_json::json!({"paragraph":i+1,"words":p.words,"status":if s.flagged{"flagged"}else{"not_flagged"},"score":s.score}));
            }
            Err(_) => segments.push(
                serde_json::json!({"paragraph":i+1,"words":p.words,"status":"unknown_vocabulary"}),
            ),
        }
    }
    if eligible == 0 {
        return Err(JsValue::from_str(
            "Tidak ada paragraf yang dapat dinilai. Gunakan 25–200 kata per paragraf.",
        ));
    }
    Ok(serde_json::json!({"model":"detectai-topic-v1","threshold":0.47677173837318837_f64,"status":"research_only","total_words":doc.words,"paragraphs":doc.paragraphs.len(),"eligible_words":eligible,"flagged_words":flagged,"percentage":flagged as f64/eligible as f64*100.,"excluded_words":doc.words-eligible,"segments":segments}).to_string())
}
