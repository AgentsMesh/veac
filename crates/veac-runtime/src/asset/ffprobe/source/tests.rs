use std::io;

use super::*;

#[test]
fn snapshot_helpers_preserve_io_context_and_reject_directories() {
    let temp = tempfile::tempdir().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    let error = match ProbeSource::capture(temp.path(), deadline) {
        Err(error) => error,
        Ok(_) => panic!("directories must not become media snapshots"),
    };
    assert!(matches!(
        error,
        ProbeError::Io {
            operation: "snapshot",
            ref path,
            ..
        } if path == temp.path()
    ));

    let missing = temp.path().join("missing");
    assert_eq!(
        readonly(&missing).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
    let translated = io_error(
        "protect snapshot",
        &missing,
        io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
    );
    assert!(matches!(
        translated,
        ProbeError::Io {
            operation: "protect snapshot",
            path,
            source,
        } if path == missing && source.kind() == io::ErrorKind::PermissionDenied
    ));
}

#[test]
fn expired_snapshot_deadline_remains_a_resource_error() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("media.bin");
    std::fs::write(&source, b"media").unwrap();
    let error = match ProbeSource::capture(&source, std::time::Instant::now()) {
        Err(error) => error,
        Ok(_) => panic!("expired capture must fail"),
    };
    assert!(matches!(
        error,
        ProbeError::ResourceLimit {
            operation: "source snapshot"
        }
    ));
}

#[test]
fn static_image_hints_are_explicit_safe_pipe_demuxers() {
    let cases = [
        ("image.bmp", "bmp_pipe"),
        ("image.dds", "dds_pipe"),
        ("image.dpx", "dpx_pipe"),
        ("image.exr", "exr_pipe"),
        ("image.jfif", "jpeg_pipe"),
        ("image.jpe", "jpeg_pipe"),
        ("image.jpeg", "jpeg_pipe"),
        ("image.jpg", "jpeg_pipe"),
        ("image.jls", "jpegls_pipe"),
        ("image.jxl", "jpegxl_pipe"),
        ("image.PNG", "png_pipe"),
        ("image.psd", "psd_pipe"),
        ("image.tif", "tiff_pipe"),
        ("image.tiff", "tiff_pipe"),
        ("image.webp", "webp_pipe"),
    ];
    let safe_formats = crate::input_policy::string_arguments()[3].clone();
    for (path, expected) in cases {
        assert_eq!(static_image_demuxer(Path::new(path)), Some(expected));
        assert!(safe_formats.split(',').any(|value| value == expected));
        assert_ne!(expected, "image2");
    }
    for path in ["image", "image.mp4", "image.m3u8", "image.image2"] {
        assert_eq!(static_image_demuxer(Path::new(path)), None);
    }
}

#[test]
fn captured_png_forces_its_demuxer_immediately_before_input() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("frame.PNG");
    std::fs::write(&path, b"snapshot bytes").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    let source = ProbeSource::capture(&path, deadline).unwrap();
    let mut arguments = crate::input_policy::os_arguments().to_vec();
    source.append_input_arguments(&mut arguments);

    assert_eq!(arguments[4], "-f");
    assert_eq!(arguments[5], "png_pipe");
    assert_eq!(arguments[6], "-i");
    assert_eq!(arguments[7], source.path);
}

#[test]
fn captured_non_image_keeps_safe_auto_detection() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("video.mp4");
    std::fs::write(&path, b"snapshot bytes").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    let source = ProbeSource::capture(&path, deadline).unwrap();
    let mut arguments = Vec::new();
    source.append_input_arguments(&mut arguments);

    assert_eq!(arguments[0], "-i");
    assert_eq!(arguments[1], source.path);
}
