use std::path::Path;

use veac_ir::{HashAlgorithm, MediaIdentity, RationalTime};

use super::*;

#[test]
fn limits_reject_invalid_policies_and_large_frames() {
    let limits = ObservationLimits {
        max_width: 0,
        ..ObservationLimits::default()
    };
    assert!(limits.validate().is_err());
    let limits = ObservationLimits::default();
    assert_eq!(limits.frame_bytes(2, 3, 4).unwrap(), 24);
    assert!(limits.frame_bytes(9_000, 1, 4).is_err());
    assert!(limits.frame_bytes(8_000, 8_000, 4).is_err());
}

#[test]
fn frame_commands_are_closed_and_deterministic() {
    let time = RationalTime::new(1, 3).unwrap();
    let rgb = command::frame(Path::new("input.mov"), 2, time, FramePixelFormat::Rgb8).unwrap();
    assert!(rgb
        .windows(2)
        .any(|pair| pair == ["-vf", "trim=start=0.333333333,showinfo"]));
    assert!(rgb.windows(2).any(|pair| pair == ["-map", "0:v:2"]));
    assert!(rgb.windows(2).any(|pair| pair == ["-pix_fmt", "rgb24"]));
    assert!(rgb.iter().any(|value| value == "-protocol_whitelist"));

    let alpha = command::frame(
        Path::new("input.mov"),
        0,
        RationalTime::zero(1).unwrap(),
        FramePixelFormat::Alpha16,
    )
    .unwrap();
    assert!(alpha
        .windows(2)
        .any(|pair| pair == ["-vf", "trim=start=0.0,alphaextract,showinfo"]));
    assert_eq!(
        command::decode(Path::new("input.mov"), 1)
            .unwrap()
            .last()
            .unwrap(),
        "-"
    );
}

#[test]
fn model_formats_and_early_request_validation_are_stable() {
    assert_eq!(FramePixelFormat::Rgba8.ffmpeg_name(), "rgba");
    assert_eq!(FramePixelFormat::Rgba8.bytes_per_pixel(), 4);
    let observer = MediaObserver::default();
    let request = FrameRequest {
        source: ObservationSource {
            path: "missing.mov".into(),
            identity: MediaIdentity {
                algorithm: HashAlgorithm::Sha256,
                digest: "0".repeat(64),
            },
            video_stream: None,
        },
        time: RationalTime::new(-1, 1).unwrap(),
        pixel_format: FramePixelFormat::Rgba8,
    };
    assert!(observer
        .frame(&request)
        .unwrap_err()
        .message
        .contains("non-negative"));
}
