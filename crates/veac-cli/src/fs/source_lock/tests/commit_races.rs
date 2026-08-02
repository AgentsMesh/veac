use super::SourceGraphLock;

#[test]
fn final_check_rejects_a_replaced_module_parent() {
    let temp = tempfile::tempdir().unwrap();
    let parts = temp.path().join("parts");
    let moved = temp.path().join("moved");
    std::fs::create_dir(&parts).unwrap();
    std::fs::write(parts.join("brand.veac"), "before").unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();

    let error = lock
        .commit_module_with(
            temp.path(),
            "parts/brand.veac",
            "before",
            "after",
            || {
                std::fs::rename(&parts, &moved).unwrap();
                std::fs::create_dir(&parts).unwrap();
                std::fs::write(parts.join("brand.veac"), "foreign").unwrap();
            },
            || {},
        )
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    assert_eq!(
        std::fs::read_to_string(parts.join("brand.veac")).unwrap(),
        "foreign"
    );
    assert_eq!(
        std::fs::read_to_string(moved.join("brand.veac")).unwrap(),
        "before"
    );
}

#[test]
fn final_check_rejects_target_bytes_changed_in_place() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("main.veac");
    std::fs::write(&source, "before").unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();

    let error = lock
        .commit_module_with(
            temp.path(),
            "main.veac",
            "before",
            "after",
            || {
                std::fs::write(&source, "external").unwrap();
            },
            || {},
        )
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    assert_eq!(std::fs::read_to_string(source).unwrap(), "external");
}

#[test]
fn final_check_rejects_a_same_content_target_replacement() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("main.veac");
    let displaced = temp.path().join("displaced.veac");
    std::fs::write(&source, "before").unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();

    let error = lock
        .commit_module_with(
            temp.path(),
            "main.veac",
            "before",
            "after",
            || {
                std::fs::rename(&source, &displaced).unwrap();
                std::fs::write(&source, "before").unwrap();
            },
            || {},
        )
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    assert!(error.to_string().contains("identity or mode changed"));
    assert_eq!(std::fs::read_to_string(source).unwrap(), "before");
    assert_eq!(std::fs::read_to_string(displaced).unwrap(), "before");
}

#[test]
fn final_check_rejects_a_replaced_lock_inode() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("main.veac");
    let lock_path = temp.path().join(".veac-source.lock");
    std::fs::write(&source, "before").unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();

    let error = lock
        .commit_module_with(
            temp.path(),
            "main.veac",
            "before",
            "after",
            || {
                std::fs::remove_file(&lock_path).unwrap();
                std::fs::write(&lock_path, "replacement").unwrap();
            },
            || {},
        )
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    assert_eq!(std::fs::read_to_string(source).unwrap(), "before");
}

#[test]
fn post_publish_lock_replacement_reports_commit_uncertainty() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("main.veac");
    let lock_path = temp.path().join(".veac-source.lock");
    std::fs::write(&source, "before").unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();

    let error = lock
        .commit_module_with(
            temp.path(),
            "main.veac",
            "before",
            "after",
            || {},
            || {
                std::fs::remove_file(&lock_path).unwrap();
                std::fs::write(&lock_path, "replacement").unwrap();
            },
        )
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "WRITE_COMMIT_UNCERTAIN");
    assert_eq!(std::fs::read_to_string(source).unwrap(), "after");
}
