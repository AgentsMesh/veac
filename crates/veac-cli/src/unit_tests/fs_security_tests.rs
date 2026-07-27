#[cfg(unix)]
#[test]
fn atomic_write_ignores_predictable_temps_and_preserves_replaced_stage_entries() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("value.txt");
    let predictable = temp
        .path()
        .join(format!(".value.txt.veac-tmp-{}-0", std::process::id()));
    std::fs::write(&predictable, b"sentinel").unwrap();
    crate::fs::atomic_write(&destination, "first").unwrap();
    assert_eq!(std::fs::read(&predictable).unwrap(), b"sentinel");

    let external = temp.path().join("external");
    std::fs::write(&external, b"external").unwrap();
    let mut replacement = None;
    let error = crate::fs::atomic_write_with(&destination, "second", |stage| {
        let stage = stage.to_owned();
        std::fs::remove_file(&stage).unwrap();
        symlink(&external, &stage).unwrap();
        replacement = Some(stage);
    })
    .unwrap_err();
    let replacement = replacement.unwrap();

    assert!(error.to_string().contains("WRITE_FAILED"));
    assert!(std::fs::symlink_metadata(replacement)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(std::fs::read(external).unwrap(), b"external");
    assert_eq!(std::fs::read(destination).unwrap(), b"first");
}

#[test]
fn atomic_write_distinguishes_an_error_after_the_commit_point() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("value.txt");
    std::fs::write(&destination, b"old").unwrap();
    let mut replacement_directory = None;

    let error = crate::fs::atomic_write_with(&destination, "new", |payload| {
        let visible = payload.parent().unwrap().to_owned();
        let moved = temp.path().join("moved-stage");
        std::fs::rename(&visible, moved).unwrap();
        std::fs::create_dir(&visible).unwrap();
        std::fs::write(visible.join("foreign"), b"foreign").unwrap();
        replacement_directory = Some(visible);
    })
    .unwrap_err();

    assert!(error.to_string().contains("WRITE_COMMIT_UNCERTAIN"));
    assert!(error.to_string().contains("do not retry blindly"));
    assert_eq!(std::fs::read(destination).unwrap(), b"new");
    assert_eq!(
        std::fs::read(replacement_directory.unwrap().join("foreign")).unwrap(),
        b"foreign"
    );
}
