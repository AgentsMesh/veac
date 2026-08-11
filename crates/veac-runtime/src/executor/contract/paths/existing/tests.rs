use super::{validate_package, validate_static};

#[test]
fn inspection_failures_preserve_static_and_package_path_context() {
    let temp = tempfile::tempdir().unwrap();
    let blocking_file = temp.path().join("blocking-file");
    std::fs::write(&blocking_file, b"content").unwrap();
    let unreachable = blocking_file.join("output");

    let static_error = validate_static(&unreachable).unwrap_err();
    assert!(static_error
        .message
        .contains("cannot inspect existing backend output"));
    assert!(static_error.message.contains("output"));

    let package_error = validate_package(&unreachable).unwrap_err();
    assert!(package_error
        .message
        .contains("cannot inspect existing backend package"));
    assert!(package_error.message.contains("output"));
}
