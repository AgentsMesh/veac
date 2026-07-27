use std::io;
use std::path::Path;

use super::io_error;

#[test]
fn write_io_errors_retain_the_operation_and_path() {
    let error = io_error(
        "write temporary output",
        Path::new("result.json"),
        io::Error::other("disk unavailable"),
    );

    let rendered = error.to_string();
    assert!(rendered.contains("WRITE_FAILED"));
    assert!(rendered.contains("write temporary output result.json"));
    assert!(rendered.contains("disk unavailable"));
}
