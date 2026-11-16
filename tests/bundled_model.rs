use nusa_ai_detector::model::Model;
#[test]
fn bundled_weights_match_python_reference_scores() {
    let model = Model::from_json(include_bytes!("../models/detectai-topic-v1.json")).unwrap();
    let cases: serde_json::Value = serde_json::from_str(include_str!("model-cases.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let actual = model.infer(case["text"].as_str().unwrap()).unwrap();
        assert!((actual.score - case["score"].as_f64().unwrap()).abs() < 1e-12);
    }
}
