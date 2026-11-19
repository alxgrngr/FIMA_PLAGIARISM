use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=NUSA_MODEL_PROFILE");
    println!("cargo:rerun-if-env-changed=NUSA_JOURNAL_MODEL_PATH");
    let profile = env::var("NUSA_MODEL_PROFILE").unwrap_or_else(|_| "detectai-topic-v1".into());
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory"));
    let source = match profile.as_str() {
        "detectai-topic-v1" => root.join("models/detectai-topic-v1.json"),
        "journal-pilot-v1" => {
            let path = env::var_os("NUSA_JOURNAL_MODEL_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from("training/runs/journal-three-pilot-2026-11-19/journal-pilot.json")
                });
            root.join(path)
        }
        _ => panic!("Unknown NUSA_MODEL_PROFILE: {profile}"),
    };
    println!("cargo:rerun-if-changed={}", source.display());
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    fs::copy(&source, out.join("bundled-model.json")).unwrap_or_else(|error| {
        panic!("Cannot bundle {profile} from {}: {error}", source.display())
    });
    println!("cargo:rustc-env=NUSA_BUNDLED_MODEL_ID={profile}");
}
