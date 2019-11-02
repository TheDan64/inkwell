extern crate cargo_toml;

use cargo_toml::Manifest;

use std::error::Error;
use std::path::Path;

fn main() {
    let (manifest, manifest_path) = get_manifest().expect("Unable to load Cargo manifest!");
    let features = manifest.features
        .keys()
        .filter(|feature| feature.as_str().starts_with("llvm"))
        .cloned()
        .collect::<Vec<_>>();

    let features_csv = features.as_slice().join(",");

    // Exports a comma-separated feature list in the environment during compilation
    // Can be fetched with `env!("INKWELL_FEATURES")`
    println!("cargo:rustc-env=INKWELL_FEATURES={}", features_csv);
    if let Some(path) = manifest_path {
        println!("cargo:rerun-if-changed={}", path);
    }
}

/// If we're developing locally we should use the local manifest
/// so that we don't have to duplicate LLVM version info. If we're publishing
/// then grab the manifest from online since we need internet access to publish anyway.
fn get_manifest() -> Result<(Manifest, Option<&'static str>), Box<dyn Error>> {
    const MANIFEST_PATH: &str = "../Cargo.toml";

    Ok(match Manifest::from_path(Path::new(MANIFEST_PATH)) {
        Ok(m) => (m, Some(MANIFEST_PATH)),
        Err(_) => {
            let manifest_str = reqwest::get("https://raw.githubusercontent.com/TheDan64/inkwell/master/Cargo.toml")?.text()?;
            let manifest = Manifest::from_str(&manifest_str)?;

            (manifest, None)
        },
    })
}
