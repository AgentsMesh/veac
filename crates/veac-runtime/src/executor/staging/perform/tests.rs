use super::staging_error;

#[test]
fn sidecar_staging_errors_keep_io_context() {
    let error = staging_error(std::io::Error::other("fixture failure"));
    assert!(error
        .message
        .contains("cannot prepare FFmpeg sidecar staging"));
    assert!(error.message.contains("fixture failure"));
}
