//! Local model parity check; never logs input text.
use nusa_ai_detector::model::Model;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("Usage: score MODEL.json TEXT.txt".into());
    }
    let model = Model::from_json(&std::fs::read(&args[1])?)?;
    let score = model.infer(&std::fs::read_to_string(&args[2])?)?;
    println!("{:.15}", score.score);
    Ok(())
}
