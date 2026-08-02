use super::super::{enumerate_passlogs, enumerate_pattern};

#[test]
fn dynamic_enumeration_validates_patterns_and_sorts_matches() {
    let temp = tempfile::tempdir().unwrap();
    assert!(enumerate_pattern(&temp.path().join("frame.png"))
        .unwrap_err()
        .message
        .contains("exactly one %d"));
    assert!(enumerate_pattern(&temp.path().join("frame-%d-%d.png"))
        .unwrap_err()
        .message
        .contains("exactly one %d"));

    for name in ["frame-10.png", "frame-2.png", "frame-x.png", "other-1.png"] {
        std::fs::write(temp.path().join(name), b"frame").unwrap();
    }
    let paths = enumerate_pattern(&temp.path().join("frame-%d.png")).unwrap();
    assert_eq!(
        paths,
        vec![
            temp.path().join("frame-10.png"),
            temp.path().join("frame-2.png")
        ]
    );

    for name in ["frame-0001.png", "frame-0010.png"] {
        std::fs::write(temp.path().join(name), b"frame").unwrap();
    }
    assert_eq!(
        enumerate_pattern(&temp.path().join("frame-%04d.png")).unwrap(),
        vec![
            temp.path().join("frame-0001.png"),
            temp.path().join("frame-0010.png")
        ]
    );

    for name in ["pass-0.log", "pass-0.log.mbtree", "pass-not-log"] {
        std::fs::write(temp.path().join(name), b"pass").unwrap();
    }
    assert_eq!(
        enumerate_passlogs(&temp.path().join("pass")).unwrap().len(),
        2
    );
}

#[test]
fn matching_dynamic_non_file_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir(temp.path().join("frame-1.png")).unwrap();
    assert!(enumerate_pattern(&temp.path().join("frame-%d.png"))
        .unwrap_err()
        .message
        .contains("not a regular file"));
}

#[cfg(unix)]
#[test]
fn unreadable_output_directory_reports_inspection_failure() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let directory = temp.path().join("unreadable");
    std::fs::create_dir(&directory).unwrap();
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o0)).unwrap();
    let result = enumerate_pattern(&directory.join("frame-%d.png"));
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(result
        .unwrap_err()
        .message
        .contains("cannot inspect render outputs"));
}
