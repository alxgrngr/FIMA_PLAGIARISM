//! CPU-only research inference. The bundled model is available only in opt-in research mode.
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    schema_version: u32,
    status: String,
    tokenizer: String,
    vocabulary: HashMap<String, usize>,
    idf: Vec<f64>,
    weights: Vec<f64>,
    bias: f64,
    threshold: f64,
    calibrated: bool,
}
#[derive(Debug)]
pub struct Score {
    /// Uncalibrated logistic score, not an authorship probability.
    pub score: f64,
    pub flagged: bool,
    pub vocabulary_coverage: f64,
}
impl Model {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > 2_000_000 {
            return Err("Model exceeds size limit");
        }
        let m: Self = serde_json::from_slice(bytes).map_err(|_| "Invalid model JSON")?;
        let n = m.vocabulary.len();
        if m.schema_version != 1
            || m.status != "research_only"
            || m.tokenizer != "ascii_lower_v1"
            || m.calibrated
        {
            return Err("Unsupported model metadata");
        }
        if n == 0
            || n > 10_000
            || m.idf.len() != n
            || m.weights.len() != n
            || !m.bias.is_finite()
            || !m.threshold.is_finite()
            || !(0.0..=1.0).contains(&m.threshold)
        {
            return Err("Invalid dimensions or threshold");
        }
        if m.vocabulary.values().copied().collect::<BTreeSet<_>>() != (0..n).collect()
            || m.vocabulary
                .keys()
                .any(|s| s.is_empty() || !s.bytes().all(|b| b.is_ascii_lowercase()))
            || m.weights.iter().any(|v| !v.is_finite() || v.abs() > 1e6)
            || m.idf
                .iter()
                .any(|v| !v.is_finite() || *v <= 0.0 || *v > 100.0)
        {
            return Err("Invalid vocabulary or coefficients");
        }
        Ok(m)
    }
    pub fn infer(&self, text: &str) -> Result<Score, &'static str> {
        if text.chars().count() > crate::MAX_CHARS {
            return Err("Text exceeds size limit");
        }
        let lower = if text
            .bytes()
            .all(|b| b.is_ascii() && !b.is_ascii_uppercase())
        {
            Cow::Borrowed(text)
        } else {
            Cow::Owned(text.to_lowercase())
        };
        let mut counts = BTreeMap::<usize, usize>::new();
        let mut total = 0;
        let mut known = 0;
        for word in lower
            .split(|c: char| !c.is_ascii_lowercase())
            .filter(|s| !s.is_empty())
        {
            total += 1;
            if let Some(i) = self.vocabulary.get(word) {
                *counts.entry(*i).or_default() += 1;
                known += 1;
            }
        }
        if known == 0 {
            return Err("No known vocabulary: abstain");
        }
        let features: Vec<_> = counts
            .into_iter()
            .map(|(i, c)| (i, (1.0 + (c as f64).ln()) * self.idf[i]))
            .collect();
        let norm = features.iter().map(|(_, v)| v * v).sum::<f64>().sqrt();
        let logit = self.bias
            + features
                .iter()
                .map(|(i, v)| self.weights[*i] * v / norm)
                .sum::<f64>();
        let score = 1.0 / (1.0 + (-logit.clamp(-60.0, 60.0)).exp());
        Ok(Score {
            score,
            flagged: score >= self.threshold,
            vocabulary_coverage: known as f64 / total as f64,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const FIXTURE: &str = r#"{"schema_version":1,"status":"research_only","tokenizer":"ascii_lower_v1","vocabulary":{"halo":0,"dunia":1},"idf":[1,1],"weights":[1,-1],"bias":0,"threshold":0.6,"calibrated":false}"#;
    #[test]
    fn inference_math() {
        let m = Model::from_json(FIXTURE.as_bytes()).unwrap();
        assert!((m.infer("HALO dunia").unwrap().score - 0.5).abs() < 1e-12);
        assert!(m.infer("halo").unwrap().flagged);
    }
    #[test]
    fn abstain() {
        assert!(Model::from_json(FIXTURE.as_bytes())
            .unwrap()
            .infer("unknown")
            .is_err());
    }
    #[test]
    fn metadata() {
        assert!(
            Model::from_json(FIXTURE.replace("research_only", "production").as_bytes()).is_err()
        );
    }
    #[test]
    fn duplicate_index() {
        assert!(
            Model::from_json(FIXTURE.replace("\"dunia\":1", "\"dunia\":0").as_bytes()).is_err()
        );
    }
    #[test]
    fn limits() {
        assert!(Model::from_json(FIXTURE.as_bytes())
            .unwrap()
            .infer(&"a".repeat(crate::MAX_CHARS + 1))
            .is_err());
    }
}
