use std::fs;

use tempfile::tempdir;

use super::{empty_entry, entry, error_code};
use crate::program::{compile_path, compile_source};

#[test]
fn constants_support_forward_references_and_exact_units() {
    let source = entry(
        "const time total = base + 500ms;\nconst time base = 500ms;",
        r#"sequence main {
    layer visual content {
      item solid {
        source generated solid { color #000000ff; }
        record { at 0s; duration ${total}; }
      }
    }
  }"#,
    );
    let compiled = compile_source(&source).unwrap();
    assert!(compiled.expanded_source().contains("duration 1s;"));
}

#[test]
fn constant_cycles_have_a_specific_diagnostic() {
    let source = empty_entry("const time first = second;\nconst time second = first;");
    assert_eq!(error_code(&source), "PROGRAM_CONST_CYCLE");
}

#[test]
fn imports_expose_only_exported_symbols() {
    let temp = tempdir().unwrap();
    let module = "module { const time hidden = 1s; export const time shown = 2s; }";
    fs::write(temp.path().join("values.veac"), module).unwrap();
    let good = entry(
        "import \"./values.veac\" as values;",
        r#"sequence main { layer visual content { item sample {
    source generated transparent;
    record { at 0s; duration ${values.shown}; }
  } } }"#,
    );
    fs::write(temp.path().join("main.veac"), good).unwrap();
    assert!(compile_path(&temp.path().join("main.veac")).is_ok());
    let bad = entry(
        "import \"./values.veac\" as values;\nconst time leak = values.hidden;",
        "sequence main {}",
    );
    fs::write(temp.path().join("main.veac"), bad).unwrap();
    let error = compile_path(&temp.path().join("main.veac")).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_CONST_EXPRESSION");
}

#[test]
fn module_cycles_report_the_exact_chain() {
    let temp = tempdir().unwrap();
    fs::write(
        temp.path().join("a.veac"),
        "module { import \"./b.veac\" as b; }",
    )
    .unwrap();
    fs::write(
        temp.path().join("b.veac"),
        "module { import \"./a.veac\" as a; }",
    )
    .unwrap();
    fs::write(
        temp.path().join("main.veac"),
        empty_entry("import \"./a.veac\" as a;"),
    )
    .unwrap();
    let error = compile_path(&temp.path().join("main.veac")).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_IMPORT_CYCLE");
    assert!(error.as_slice()[0]
        .message
        .contains("a.veac -> b.veac -> a.veac"));
}
