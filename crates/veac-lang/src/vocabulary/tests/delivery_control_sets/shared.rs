use crate::vocabulary::control_uses::delivery as c;

use super::super::super::{language_spec, ControlWord, VocabularyCategory};
use super::super::legacy_surface::assert_control_absent;

#[test]
fn shared_delivery_spellings_do_not_publish_legacy_uses() {
    let spec = language_spec();
    for controls in shared_groups() {
        for control in controls {
            assert_control_absent(&spec.vocabulary, *control);
        }
    }
}

#[test]
fn delivery_data_and_closed_values_are_not_contextual_controls() {
    for value in [
        "rendition-hd",
        "dialogue",
        "stream.m3u8",
        "4.1",
        "h264",
        "capped",
        "fast-start",
    ] {
        assert_eq!(ControlWord::parse(value), None, "{value}");
        if let Some(entry) = language_spec().vocabulary.lookup(value) {
            assert!(!entry
                .uses
                .iter()
                .any(|usage| { usage.category == VocabularyCategory::ContextualControl }));
        }
    }
    let positions = include_str!("../../enums/position.rs");
    assert!(!positions.contains("ArtifactAudioEncodingMember"));
    assert!(!positions.contains("ArtifactCanvasSeparator"));
}

fn shared_groups() -> Vec<&'static [crate::vocabulary::ControlUse]> {
    vec![
        &[c::SEQUENCE_FIELD],
        &[
            c::ARTIFACT_TARGET_FIELD,
            c::AVERAGE_TARGET_FIELD,
            c::CAPPED_TARGET_FIELD,
            c::HLS_CAPPED_TARGET_FIELD,
        ],
        &[
            c::CAPTION_SIDECAR_SOURCE_FIELD,
            c::AUDIO_STEM_SOURCE_FIELD,
            c::AUDIO_FILE_SOURCE_FIELD,
            c::HLS_AUDIO_SOURCE_FIELD,
        ],
        &[
            c::IMAGE_SEQUENCE_ENCODE_FIELD,
            c::CAPTION_SIDECAR_ENCODE_FIELD,
            c::AUDIO_STEM_ENCODE_FIELD,
            c::SCOPE_ENCODE_FIELD,
            c::AUDIO_FILE_ENCODE_FIELD,
            c::ANIMATED_IMAGE_ENCODE_FIELD,
            c::STILL_IMAGE_ENCODE_FIELD,
            c::HLS_AUDIO_ENCODE_FIELD,
            c::HLS_RENDITION_ENCODE_FIELD,
        ],
        &[
            c::VIDEO_AUDIO_SAMPLE_RATE_FIELD,
            c::AUDIO_STEM_SAMPLE_RATE_FIELD,
            c::MP3_SAMPLE_RATE_FIELD,
            c::HLS_AUDIO_SAMPLE_RATE_FIELD,
        ],
        &[
            c::VIDEO_AUDIO_CHANNEL_LAYOUT_FIELD,
            c::AUDIO_STEM_CHANNEL_LAYOUT_FIELD,
            c::MP3_CHANNEL_LAYOUT_FIELD,
            c::HLS_AUDIO_CHANNEL_LAYOUT_FIELD,
        ],
        &[c::VIDEO_MUX_AUDIO_FIELD, c::HLS_AUDIO_FIELD],
        &[c::VIDEO_COLOR_SPACE_FIELD, c::HLS_VIDEO_COLOR_SPACE_FIELD],
        &[
            c::RASTER_CANVAS_BY,
            c::SCOPE_CANVAS_BY,
            c::HLS_RENDITION_CANVAS_BY,
        ],
    ]
}
