use std::fs::File;

use tempfile::tempdir;

#[test]
fn utf8_reads_accept_the_exact_active_limit_and_reject_oversize_input() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("input.txt");
    std::fs::write(&path, b"four").unwrap();

    assert_eq!(
        crate::fs::read_utf8_bounded(&path, "input", 4).unwrap(),
        "four"
    );
    let error = crate::fs::read_utf8_bounded(&path, "input", 3).unwrap_err();
    assert!(error.to_string().contains("READ_FAILED"));
    assert!(error.to_string().contains("3 byte read limit"));
}

#[test]
fn default_utf8_read_rejects_a_sparse_file_above_the_hard_limit() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("oversize.txt");
    File::create(&path)
        .unwrap()
        .set_len(veac_artifact::MAX_IN_MEMORY_ARTIFACT_BYTES + 1)
        .unwrap();

    let error = crate::fs::read_utf8(&path, "input").unwrap_err();
    assert!(error.to_string().contains("READ_FAILED"));
    assert!(error.to_string().contains("byte read limit"));
}

#[test]
fn explicit_utf8_limit_cannot_disable_or_relax_the_hard_cap() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("input.txt");
    std::fs::write(&path, b"value").unwrap();

    for limit in [0, veac_artifact::MAX_IN_MEMORY_ARTIFACT_BYTES + 1] {
        let error = crate::fs::read_utf8_bounded(&path, "input", limit).unwrap_err();
        assert!(error.to_string().contains("READ_FAILED"));
        assert!(error.to_string().contains("outside the supported policy"));
    }
}

#[test]
fn utf8_reads_reject_non_regular_inputs() {
    let temp = tempdir().unwrap();
    let error = crate::fs::read_utf8(temp.path(), "input").unwrap_err();
    assert!(error.to_string().contains("READ_FAILED"));
    assert!(error.to_string().contains("regular non-symlink file"));
}

#[test]
fn bounded_reads_reject_invalid_utf8_after_safe_file_verification() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("invalid.txt");
    std::fs::write(&path, [0xff]).unwrap();

    let error = crate::fs::read_utf8_bounded(&path, "input", 1).unwrap_err();
    assert!(error.to_string().contains("READ_FAILED"));
    assert!(error.to_string().contains("as UTF-8"));
}

#[cfg(unix)]
#[test]
fn utf8_reads_do_not_follow_symbolic_links() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let target = temp.path().join("target.txt");
    let link = temp.path().join("link.txt");
    std::fs::write(&target, b"secret").unwrap();
    symlink(&target, &link).unwrap();

    let error = crate::fs::read_utf8(&link, "input").unwrap_err();
    assert!(error.to_string().contains("READ_FAILED"));
    assert!(error.to_string().contains("symbolic link"));
}
