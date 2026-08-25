use super::io_error;

#[test]
fn package_store_io_diagnostic_keeps_the_store_path() {
    let error = io_error(
        std::path::Path::new("/packages"),
        std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
    );
    assert_eq!(error.diagnostics()[0].code, "LANG_PACKAGE_STORE_IO");
    assert!(error.diagnostics()[0].message.contains("/packages"));
    assert!(error.diagnostics()[0].message.contains("denied"));
}
