use veac_codegen::emitter::emit_all;
use veac_plan::canonical::*;

use super::delivery_extended_support::file;
use super::support::{bindings, fixture, resolved, time};

#[test]
fn mp3_rate_families_fail_closed_after_plan_mutation() {
    for (sample_rate_hz, bitrate_bps) in [
        (8_000, 320_000),
        (48_000, 8_000),
        (48_000, 9_000),
        (96_000, 192_000),
    ] {
        let kind = DeliverableKind::AudioFile(AudioFile {
            source: AudioMixSource::Master,
            encoding: AudioFileEncoding::Mp3(Mp3Encoding {
                bitrate_bps,
                sample_rate_hz,
                channel_layout: AudioChannelLayout::Stereo,
            }),
        });
        assert_code(kind, "bad.mp3", "PLAN_AUDIO_FILE_INVALID");
    }
}

#[test]
fn gif_and_still_invalid_boundaries_fail_before_emission() {
    for count in [0, 1] {
        assert_code(
            DeliverableKind::AnimatedImage(AnimatedImage::Gif(GifAnimation {
                playback: GifPlayback::Times { count },
                dither: GifDither::None,
            })),
            "bad.gif",
            "PLAN_ANIMATED_IMAGE_INVALID",
        );
    }
    assert_code(
        DeliverableKind::StillImage(StillImage {
            frame: FrameSelection::Containing { at: time(3_600) },
            encoding: ImageFormat::Png,
        }),
        "bad.png",
        "PLAN_STILL_IMAGE_INVALID",
    );
    assert_code(
        DeliverableKind::StillImage(StillImage {
            frame: FrameSelection::Containing { at: time(300) },
            encoding: ImageFormat::Png,
        }),
        "bad.jpg",
        "PLAN_STILL_IMAGE_INVALID",
    );
}

fn assert_code(kind: DeliverableKind, name: &str, code: &str) {
    let mut plan = resolved(&fixture());
    plan.output.deliverables = vec![file("dlv_invalid", name, kind)];
    let error = emit_all(&plan, &bindings(&plan)).unwrap_err();
    assert!(
        error.diagnostics().iter().any(|value| value.code == code),
        "missing {code}: {:?}",
        error.diagnostics()
    );
}
