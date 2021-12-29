// Copyright 2021 Chay Nabors.

use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=\"config.json\"");
    println!("cargo:rerun-if-env-changed=\"OUT_DIR\"");

    let config = Path::new("config.json");

    if config.exists() {
        let manifest_dir_string = env::var("OUT_DIR").unwrap();
        let output_path = Path::new(&manifest_dir_string).join("config.json");
        std::fs::copy(config, output_path).unwrap();
    }
}
