use std::{fs, path::PathBuf};

use veac_lang::program::expression::UnitSuffix;

#[test]
fn expression_rendering_consumes_unit_descriptors() {
    let relative = "src/program/expression/value/render.rs";
    let source = read(relative);
    assert!(source.contains("UnitSuffix::TIME"));
    for unit in UnitSuffix::EXECUTABLE_EXPRESSION {
        assert!(!source.contains(&format!(r#""{}""#, unit.as_str())));
    }
}

fn read(relative: &str) -> String {
    let path = manifest_dir().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
