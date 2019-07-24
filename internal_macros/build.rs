extern crate cargo_toml;

use cargo_toml::Manifest;

use std::path::Path;

fn main() {
    let manifest_path = "../Cargo.toml";
    let manifest = Manifest::from_path(Path::new(manifest_path))
        .expect("Unable to load Cargo manifest!");

    let features = manifest.features
        .keys()
        .filter(|feature| feature.as_str().starts_with("llvm"))
        .cloned()
        .collect::<Vec<_>>();

    let features_csv = features.as_slice().join(",");

    // Exports a comma-separated feature list in the environment during compilation
    // Can be fetched with `env!("INKWELL_FEATURES")`
    println!("cargo:rustc-env=INKWELL_FEATURES={}", features_csv);
    println!("cargo:rerun-if-changed={}", manifest_path);
}
