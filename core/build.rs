use std::env;
use cbindgen::Language;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(Language::C)
        .with_parse_deps(false)
        .include_item("Event")
        .generate()
        .expect("unable to generate bindings")
        .write_to_file("../mobile/simulator/SkjuSimulator/eventspod.h");
}
