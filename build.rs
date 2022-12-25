extern crate windres;

use windres::Build;

fn main() {
    Build::new().compile("manifest\\Resource.rc").unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=manifest\\Resource.rc");
    println!("cargo:rerun-if-changed=manifest\\manifest.json");
}