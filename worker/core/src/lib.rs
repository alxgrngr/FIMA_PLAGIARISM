use nusa_ai_detector::{
    documents,
    model::{Model, BUNDLED_MODEL_ID},
    paragraph_lines, MODEL_PARAGRAPH_WORDS,
};
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;
use wasm_bindgen::prelude::*;
static MODEL: OnceLock<Result<Model, &'static str>> = OnceLock::new();
fn model() -> Result<&'static Model, JsValue> {
    MODEL
        .get_or_init(Model::bundled)
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
    let model = model()?;
    let mut total_words = 0usize;
    let mut eligible = 0usize;
    let mut flagged = 0usize;
    let mut segments = Vec::new();
    for (i, paragraph) in paragraph_lines(&text).enumerate() {
        let words = paragraph.split_whitespace().count();
        total_words += words;
        if !MODEL_PARAGRAPH_WORDS.contains(&words) {
            segments.push(
                serde_json::json!({"paragraph":i+1,"words":words,"status":"excluded_length"}),
            );
            continue;
        }
        let score = if paragraph.is_ascii() {
            model.infer(paragraph)
        } else {
            model.infer(&paragraph.nfc().collect::<String>())
        };
        match score {
            Ok(s) => {
                eligible += words;
                if s.flagged {
                    flagged += words;
                }
                segments.push(serde_json::json!({"paragraph":i+1,"words":words,"status":if s.flagged{"flagged"}else{"not_flagged"},"score":s.score}));
            }
            Err(_) => segments.push(
                serde_json::json!({"paragraph":i+1,"words":words,"status":"unknown_vocabulary"}),
            ),
        }
    }
    if eligible == 0 {
        return Err(JsValue::from_str(
            "Tidak ada paragraf yang dapat dinilai. Gunakan 25–200 kata per paragraf.",
        ));
    }
    Ok(serde_json::json!({"model":BUNDLED_MODEL_ID,"threshold":model.threshold(),"status":"research_only","total_words":total_words,"paragraphs":segments.len(),"eligible_words":eligible,"flagged_words":flagged,"percentage":flagged as f64/eligible as f64*100.,"excluded_words":total_words-eligible,"segments":segments}).to_string())
}
