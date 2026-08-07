use std::fs;

use tempfile::tempdir;
use veac_lang::program::build_path;

use super::support::{project_with, result_duration, validate_canonical};

const TIMING: &str = r#"module {
  export fn padded(value: time, enabled: bool) -> time {
    helper(value, enabled)
  }
  fn helper(value: time, enabled: bool) -> time {
    let doubled = value * 2.0;
    if enabled && doubled >= 2s { doubled + 250ms } else { 2s }
  }
}"#;

#[test]
fn imported_export_can_call_a_later_private_block_function() {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("timing.veac"), TIMING).unwrap();
    let source = project_with(
        "import \"./timing.veac\" as timing;",
        "timing.padded(1s, true)",
    );
    let entry = temp.path().join("main.veac");
    fs::write(&entry, source).unwrap();

    let built = build_path(&entry).unwrap();
    assert_eq!(result_duration(&built), "2250ms");
    validate_canonical(&built);
}
