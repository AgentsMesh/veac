use crate::vocabulary::control_uses::delivery as c;

use super::super::super::{ControlUse, GrammarPosition as P};

pub(super) fn groups() -> Vec<(P, &'static [ControlUse])> {
    vec![
        (P::WavEncodingMember, &[c::WAV_SAMPLE_FORMAT_FIELD]),
        (
            P::AudioStemEncodingMember,
            &[
                c::AUDIO_STEM_SAMPLE_RATE_FIELD,
                c::AUDIO_STEM_CHANNEL_LAYOUT_FIELD,
            ],
        ),
        (
            P::Mp3EncodingMember,
            &[
                c::MP3_BITRATE_FIELD,
                c::MP3_SAMPLE_RATE_FIELD,
                c::MP3_CHANNEL_LAYOUT_FIELD,
            ],
        ),
        (
            P::GifEncodingMember,
            &[c::GIF_PLAYBACK_FIELD, c::GIF_DITHER_FIELD],
        ),
        (
            P::HlsPackageMember,
            &[c::HLS_SEGMENT_DURATION_FIELD, c::HLS_AUDIO_FIELD],
        ),
        (
            P::HlsAudioMember,
            &[c::HLS_AUDIO_SOURCE_FIELD, c::HLS_AUDIO_ENCODE_FIELD],
        ),
        (
            P::HlsAudioEncodingMember,
            &[
                c::HLS_AUDIO_BITRATE_FIELD,
                c::HLS_AUDIO_SAMPLE_RATE_FIELD,
                c::HLS_AUDIO_CHANNEL_LAYOUT_FIELD,
            ],
        ),
        (
            P::HlsRenditionMember,
            &[c::HLS_RENDITION_CANVAS_FIELD, c::HLS_RENDITION_ENCODE_FIELD],
        ),
        (
            P::HlsVideoEncodingMember,
            &[
                c::HLS_VIDEO_RATE_CONTROL_FIELD,
                c::HLS_VIDEO_PROFILE_FIELD,
                c::HLS_VIDEO_LEVEL_FIELD,
                c::HLS_VIDEO_COLOR_SPACE_FIELD,
                c::HLS_VIDEO_B_FRAMES_FIELD,
            ],
        ),
        (
            P::HlsCappedRateControlMember,
            &[
                c::HLS_CAPPED_TARGET_FIELD,
                c::HLS_CAPPED_MAX_FIELD,
                c::HLS_CAPPED_BUFFER_FIELD,
            ],
        ),
    ]
}
