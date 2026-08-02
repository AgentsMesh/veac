#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;

use super::*;

#[test]
fn dry_run_with_explicit_target_needs_no_source_root_write() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &program_source());
    let batch_path = temp.path().join("read-only.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch(
            revision(&source),
            "800ms",
        ))
        .unwrap(),
    )
    .unwrap();
    let original_mode = std::fs::metadata(temp.path()).unwrap().permissions().mode();
    std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o555)).unwrap();

    let result = veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--dry-run",
            "--output",
            source.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(original_mode)).unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!temp.path().join(".veac-source.lock").exists());
}
