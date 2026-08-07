use std::collections::BTreeMap;

use tempfile::tempdir;

use super::{ensure_source_graph_unchanged, ensure_source_modules_unchanged};

const ENTRY_SOURCE: &str =
    "fn main(context: Context) -> Project { project(identifier(\"main\"), project_settings(600)) }\n";

#[test]
fn graph_check_rejects_changed_missing_and_escaping_modules() {
    let temp = tempdir().unwrap();
    let module = temp.path().join("brand.veac");
    std::fs::write(&module, "module {}\n").unwrap();
    let expected = BTreeMap::from([("brand.veac".into(), "module {}\n".into())]);
    check(temp.path(), "brand.veac", &expected).unwrap();
    assert_eq!(
        check(temp.path(), "brand.veac", &BTreeMap::new())
            .unwrap_err()
            .diagnostics()[0]
            .code,
        "SOURCE_CHANGED"
    );

    std::fs::write(&module, "module { /* changed */ }\n").unwrap();
    let changed = check(temp.path(), "brand.veac", &expected).unwrap_err();
    assert_eq!(changed.diagnostics()[0].code, "SOURCE_CHANGED");
    std::fs::remove_file(&module).unwrap();
    assert_eq!(
        check(temp.path(), "brand.veac", &expected)
            .unwrap_err()
            .diagnostics()[0]
            .code,
        "SOURCE_CHANGED"
    );

    std::fs::write(&module, "module {}\n").unwrap();
    let invalid = BTreeMap::from([
        ("brand.veac".into(), "module {}\n".into()),
        ("../outside.veac".into(), "module {}".into()),
    ]);
    assert_eq!(
        check(temp.path(), "brand.veac", &invalid)
            .unwrap_err()
            .diagnostics()[0]
            .code,
        "SOURCE_CHANGED"
    );
}

#[test]
fn graph_check_rejects_a_missing_imported_module() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    std::fs::write(&entry, ENTRY_SOURCE).unwrap();
    let expected = BTreeMap::from([
        ("main.veac".into(), ENTRY_SOURCE.into()),
        ("missing.veac".into(), "module {}\n".into()),
    ]);

    let error = check(temp.path(), "main.veac", &expected).unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    assert!(error.to_string().contains("missing.veac"));
}

#[test]
fn module_check_maps_entry_path_and_import_failures() {
    let temp = tempdir().unwrap();
    assert_source_changed(ensure_source_modules_unchanged(
        temp.path(),
        "missing.veac",
        &[],
    ));

    std::fs::write(temp.path().join("main.veac"), ENTRY_SOURCE).unwrap();
    let duplicated = ["main.veac".to_owned(), "main.veac".to_owned()];
    assert_source_changed(ensure_source_graph_unchanged(
        temp.path(),
        "main.veac",
        &duplicated,
        &veac_lang::source_edit::SourceRevision {
            source_graph_sha256: "0".repeat(64),
        },
    ));
    assert_source_changed(ensure_source_modules_unchanged(
        temp.path(),
        "main.veac",
        &[("../outside.veac", "module {}\n")],
    ));
    assert_source_changed(ensure_source_modules_unchanged(
        temp.path(),
        "main.veac",
        &[("main.veac", ENTRY_SOURCE), ("missing.veac", "module {}\n")],
    ));
}

#[test]
fn graph_check_rejects_changed_imported_module_bytes() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("main.veac"), ENTRY_SOURCE).unwrap();
    let module = temp.path().join("brand.veac");
    std::fs::write(&module, "module {}\n").unwrap();
    let expected = BTreeMap::from([
        ("main.veac".into(), ENTRY_SOURCE.into()),
        ("brand.veac".into(), "module {}\n".into()),
    ]);
    check(temp.path(), "main.veac", &expected).unwrap();

    std::fs::write(module, "module { /* changed */ }\n").unwrap();
    let error = check(temp.path(), "main.veac", &expected).unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
}

#[cfg(unix)]
#[test]
fn graph_check_rejects_a_module_symlink_outside_the_root() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let outside = tempdir().unwrap();
    let target = outside.path().join("outside.veac");
    std::fs::write(&target, "module {}\n").unwrap();
    symlink(&target, temp.path().join("brand.veac")).unwrap();
    let expected = BTreeMap::from([("brand.veac".into(), "module {}\n".into())]);
    assert_eq!(
        check(temp.path(), "brand.veac", &expected)
            .unwrap_err()
            .diagnostics()[0]
            .code,
        "SOURCE_CHANGED"
    );
}

fn check(
    root: &std::path::Path,
    entry: &str,
    expected: &BTreeMap<String, String>,
) -> crate::error::CliResult {
    let modules = expected.keys().cloned().collect::<Vec<_>>();
    let source_modules = expected
        .iter()
        .map(|(path, source)| veac_lang::source_edit::SourceModule::utf8(path, source))
        .collect::<Vec<_>>();
    let revision = veac_lang::source_edit::source_graph_revision(&source_modules).unwrap_or(
        veac_lang::source_edit::SourceRevision {
            source_graph_sha256: "0".repeat(64),
        },
    );
    ensure_source_graph_unchanged(root, entry, &modules, &revision)
}

fn assert_source_changed(result: crate::error::CliResult) {
    assert_eq!(result.unwrap_err().diagnostics()[0].code, "SOURCE_CHANGED");
}
