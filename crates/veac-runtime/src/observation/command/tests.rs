#[cfg(unix)]
#[test]
fn frame_and_decode_reject_non_utf8_paths() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

    use veac_ir::RationalTime;

    let path = PathBuf::from(OsString::from_vec(vec![0xff]));
    let frame_error = super::frame(
        &path,
        0,
        RationalTime::zero(1).unwrap(),
        super::FramePixelFormat::Rgb8,
    )
    .unwrap_err();
    let decode_error = super::decode(&path, 0).unwrap_err();

    for error in [frame_error, decode_error] {
        assert_eq!(error.kind, crate::RuntimeErrorKind::General);
        assert_eq!(error.message, "media observation path is not valid UTF-8");
    }
}
