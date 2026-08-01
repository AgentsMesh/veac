use super::*;

#[test]
fn package_destination_accepts_missing_or_directory_and_rejects_files() {
    let temp = tempfile::tempdir().unwrap();
    let canonical = std::fs::canonicalize(temp.path()).unwrap();
    let missing = temp.path().join("missing");
    assert_eq!(
        package_destination(&missing).unwrap(),
        canonical.join("missing")
    );

    let directory = temp.path().join("stream");
    std::fs::create_dir(&directory).unwrap();
    assert_eq!(
        package_destination(&directory).unwrap(),
        canonical.join("stream")
    );

    let file = temp.path().join("file");
    std::fs::write(&file, b"file").unwrap();
    assert!(package_destination(&file)
        .unwrap_err()
        .to_string()
        .contains("OUTPUT_IS_FILE"));
    assert!(destination(&directory)
        .unwrap_err()
        .to_string()
        .contains("OUTPUT_IS_DIRECTORY"));
}

#[cfg(unix)]
#[test]
fn package_destination_rejects_symlinks() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let directory = temp.path().join("stream");
    let link = temp.path().join("link");
    std::fs::create_dir(&directory).unwrap();
    symlink(&directory, &link).unwrap();
    assert!(package_destination(&link)
        .unwrap_err()
        .to_string()
        .contains("OUTPUT_SYMLINK"));
}
