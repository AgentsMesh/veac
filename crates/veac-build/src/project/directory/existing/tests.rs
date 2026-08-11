use super::{equal, matches};
use crate::{BuildErrorKind, CancellationToken};

#[test]
fn byte_comparison_handles_equal_different_and_cancelled_inputs() {
    let temp = tempfile::tempdir().unwrap();
    let left = temp.path().join("left");
    let right = temp.path().join("right");
    std::fs::write(&left, b"same").unwrap();
    std::fs::write(&right, b"same").unwrap();
    assert!(equal(&left, &right, &CancellationToken::new()).unwrap());

    std::fs::write(&right, b"longer").unwrap();
    assert!(!equal(&left, &right, &CancellationToken::new()).unwrap());
    std::fs::write(&right, b"diff").unwrap();
    assert!(!equal(&left, &right, &CancellationToken::new()).unwrap());

    let token = CancellationToken::new();
    token.cancel();
    assert_eq!(
        equal(&left, &left, &token).unwrap_err().kind(),
        BuildErrorKind::Cancelled
    );
    assert_eq!(
        equal(
            &temp.path().join("missing"),
            &right,
            &CancellationToken::new()
        )
        .unwrap_err()
        .kind(),
        BuildErrorKind::Cache
    );
}

#[test]
fn existing_directory_comparison_maps_pack_failures() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("file");
    std::fs::write(&file, b"x").unwrap();
    let error = matches(&file, &file, temp.path(), &CancellationToken::new()).unwrap_err();
    assert_eq!(error.kind(), BuildErrorKind::InvalidContract);

    let directory = temp.path().join("directory");
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(directory.join("value"), b"x").unwrap();
    let token = CancellationToken::new();
    token.cancel();
    let error = matches(&directory, &file, temp.path(), &token).unwrap_err();
    assert_eq!(error.kind(), BuildErrorKind::Cancelled);
}
