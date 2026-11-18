pub mod documents;
pub mod model;
use unicode_normalization::UnicodeNormalization;

pub const MAX_CHARS: usize = 100_000;
#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    pub start: usize,
    pub end: usize,
    pub words: usize,
    pub normalized: String,
}
#[derive(Debug, Clone)]
pub struct Analysis {
    pub paragraphs: Vec<Paragraph>,
    pub words: usize,
    pub characters: usize,
    pub lexical_diversity: f64,
}

/// Nonblank input lines, in the same order as the browser paragraph preview.
pub fn paragraph_lines(text: &str) -> impl Iterator<Item = &str> {
    text.split('\n')
        .map(str::trim)
        .filter(|line| !line.is_empty())
}
pub fn analyze(text: &str) -> Result<Analysis, &'static str> {
    let characters = text.chars().count();
    if characters > MAX_CHARS {
        return Err("Teks melebihi batas 100.000 karakter.");
    }
    if text.trim().is_empty() {
        return Err("Masukkan teks terlebih dahulu.");
    }
    let mut paragraphs = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let start = offset + line.find(trimmed).unwrap_or(0);
            paragraphs.push(Paragraph {
                start,
                end: start + trimmed.len(),
                words: trimmed.split_whitespace().count(),
                normalized: trimmed.nfc().collect(),
            });
        }
        offset += line.len();
    }
    let words = text.split_whitespace().count();
    let unique: std::collections::HashSet<_> =
        text.split_whitespace().map(str::to_lowercase).collect();
    Ok(Analysis {
        paragraphs,
        words,
        characters,
        lexical_diversity: unique.len() as f64 / words.max(1) as f64,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts() {
        let a = analyze("Halo dunia\n\nIni teks.").unwrap();
        assert_eq!(a.words, 4);
        assert_eq!(a.paragraphs.len(), 2);
    }
    #[test]
    fn positions() {
        let s = "  café\r\n\n  Indonesia 🇮🇩";
        for p in analyze(s).unwrap().paragraphs {
            assert_eq!(s[p.start..p.end].nfc().collect::<String>(), p.normalized);
        }
    }
    #[test]
    fn empty() {
        assert!(analyze(" \n").is_err());
    }
    #[test]
    fn limits() {
        assert!(analyze(&"a".repeat(MAX_CHARS + 1)).is_err());
    }
    #[test]
    fn normalization() {
        assert_eq!(analyze("e\u{301}").unwrap().paragraphs[0].normalized, "é");
    }
}
