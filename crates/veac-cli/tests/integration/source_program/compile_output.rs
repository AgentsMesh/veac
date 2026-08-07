use super::*;

fn fixture() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let entry_source = format!(
        "import \"./brand.veac\" as brand;\n{}",
        GENERATED_SOURCE.replace("during(0s, 200ms)", "during(0s, brand.duration)")
    );
    let entry = source_file(&temp, &entry_source);
    let module = temp.path().join("brand.veac");
    std::fs::write(&module, "module { export const time duration = 200ms; }\n").unwrap();
    (temp, entry, module)
}

fn assert_build_rejects(entry: &std::path::Path, output: &std::path::Path, code: &str) {
    veac()
        .args([
            "build",
            entry.to_str().unwrap(),
            "--emit-ir",
            output.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(code));
}

#[test]
fn build_cannot_overwrite_an_imported_source_module() {
    let (_temp, entry, module) = fixture();
    let original = std::fs::read(&module).unwrap();
    assert_build_rejects(&entry, &module, "OUTPUT_OVERWRITES_INPUT");
    assert_eq!(std::fs::read(module).unwrap(), original);
}

#[test]
fn build_cannot_overwrite_a_hardlink_to_an_imported_module() {
    let (temp, entry, module) = fixture();
    let alias = temp.path().join("module-alias.json");
    std::fs::hard_link(&module, &alias).unwrap();
    let original = std::fs::read(&module).unwrap();
    assert_build_rejects(&entry, &alias, "OUTPUT_OVERWRITES_INPUT");
    assert_eq!(std::fs::read(module).unwrap(), original);
}

#[cfg(unix)]
#[test]
fn build_rejects_a_symlink_output_before_writing() {
    use std::os::unix::fs::symlink;

    let (temp, entry, module) = fixture();
    let alias = temp.path().join("module-alias.json");
    symlink(&module, &alias).unwrap();
    let original = std::fs::read(&module).unwrap();
    assert_build_rejects(&entry, &alias, "OUTPUT_SYMLINK");
    assert_eq!(std::fs::read(module).unwrap(), original);
}
