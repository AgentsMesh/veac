use std::fs;

use tempfile::tempdir;
use veac_lang::program::compile_path;

const SETTINGS: &str = r#"settings {
  timebase 1/1000; canvas 320px by 180px;
  frame-rate 30fps; sample-rate 48000hz;
}"#;

#[test]
fn imported_wrong_kind_is_found_from_the_project_scope_name_index() {
    let module = r#"module {
  export preset text-layout shared {
    box-width 100px; box-height 40px; wrap word; overflow clip;
    horizontal-align center; vertical-align middle;
  }
}"#;
    let project = format!(
        r#"import "./theme.veac" as theme;
project wrong-kind {{
  {SETTINGS}
  entry sequence main;
  sequence main {{ layer visual content {{ item title {{
    source text {{
      content "title"; style {{ use text-style theme.shared; }}
      layout {{
        box-width 100px; box-height 40px; wrap word; overflow clip;
        horizontal-align center; vertical-align middle;
      }}
    }}
    record {{ at 0s; duration 1s; }}
  }} }} }}
}}"#
    );
    assert_compile_code(module, &project, "PROGRAM_PRESET_KIND_MISMATCH");
}

#[test]
fn wrong_kind_is_found_from_a_component_captured_scope_name_index() {
    let module = r#"module {
  preset text-layout shared {}
  export component sequence card {
    body {
      layer visual content { item title {
        source text {
          content "title"; style { use text-style shared; }
          layout {
            box-width 100px; box-height 40px; wrap word; overflow clip;
            horizontal-align center; vertical-align middle;
          }
        }
        record { at 0s; duration 1s; }
      } }
    }
  }
}"#;
    let project = format!(
        "import \"./theme.veac\" as theme;\nproject wrong-kind {{\n  {SETTINGS}\n  \
         entry sequence main; sequence main {{}}\n}}"
    );
    assert_compile_code(module, &project, "PROGRAM_PRESET_KIND_MISMATCH");
}

fn assert_compile_code(module: &str, project: &str, expected: &str) {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, project).unwrap();
    fs::write(temp.path().join("theme.veac"), module).unwrap();
    let error = compile_path(&entry).unwrap_err();
    assert_eq!(error.as_slice()[0].code, expected);
}
