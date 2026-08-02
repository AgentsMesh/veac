use crate::{video_settings_valid, AlphaMode, PixelFormat, VideoCodec, VideoOutput};

use super::support::video_for;

#[test]
fn prores_requires_the_only_exact_profile_pixel_contract_in_the_model() {
    let valid = video_for(VideoCodec::ProRes);
    assert!(video_settings_valid(&valid));
    for invalid in [
        VideoOutput {
            pixel_format: PixelFormat::Yuv420p,
            alpha: AlphaMode::Opaque,
            profile: None,
            ..valid.clone()
        },
        VideoOutput {
            pixel_format: PixelFormat::Yuv420p10le,
            ..valid.clone()
        },
        VideoOutput {
            profile: None,
            ..valid.clone()
        },
        VideoOutput {
            alpha: AlphaMode::Opaque,
            ..valid
        },
    ] {
        assert!(!video_settings_valid(&invalid));
    }
}
