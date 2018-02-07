extern crate toml_edit;

use std::env::{VarError, var};
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;

use toml_edit::{Array, Document, Value, value};

static ENV_VAR: &str = "INKWELL_LLVM_VERSION";

enum LlvmVersion {
    ThreeSix,
    ThreeSeven,
}

impl LlvmVersion {
    fn variants() -> [&'static str; 2] {
        ["3.6", "3.7"]
    }

    fn dependency_version(&self) -> &'static str {
        match *self {
            LlvmVersion::ThreeSix => "36",
            LlvmVersion::ThreeSeven => "37",
        }
    }

    fn feature_version(&self) -> &'static str {
        match *self {
            LlvmVersion::ThreeSix => "llvm3-6",
            LlvmVersion::ThreeSeven => "llvm3-7",
        }
    }
}

impl FromStr for LlvmVersion {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "3.6" => Ok(LlvmVersion::ThreeSix),
            "3.7" => Ok(LlvmVersion::ThreeSeven),
            _ => Err(()),
        }
    }
}

fn main() {
    let llvm_version = match var(ENV_VAR) {
        Ok(s) => s,
        Err(VarError::NotPresent) => panic!("Could not find environment variable {}.", ENV_VAR),
        Err(VarError::NotUnicode(e)) => panic!("Environment variable {} does not contain valid unicode: {:?}.", ENV_VAR, e),
    };
    let llvm_version: LlvmVersion = match llvm_version.parse() {
        Ok(v) => v,
        Err(()) => panic!("Environment variable {} does not contain one of: {:?}", ENV_VAR, LlvmVersion::variants()),
    };

    let mut file = File::open("Cargo.toml").expect("Could not find Cargo.toml");
    let mut buffer = String::new();

    file.read_to_string(&mut buffer).expect("Could not read from Cargo.toml");

    let mut toml: Document = buffer.parse().expect("Could not parse Cargo.toml");

    toml["dependencies"]["llvm-sys"] = value(llvm_version.dependency_version());

    let mut default = Array::default();

    default.push(llvm_version.feature_version());

    toml["features"]["default"] = value(Value::Array(default));

    let mut file = File::create("Cargo.toml").expect("Could not open Cargo.toml to write");

    file.write(toml.to_string().as_bytes()).expect("Could not write to Cargo.toml");
}
