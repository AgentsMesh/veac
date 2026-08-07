use crate::vocabulary::control_uses::delivery as c;

use super::super::super::{ControlUse, GrammarPosition as P};

pub(super) fn groups() -> Vec<(P, &'static [ControlUse])> {
    vec![
        (P::DeliveryMember, &[c::SEQUENCE_FIELD, c::RASTER_FIELD]),
        (
            P::DeliveryRasterMember,
            &[
                c::RASTER_CANVAS_FIELD,
                c::RASTER_FRAME_RATE_FIELD,
                c::RASTER_CAPTIONS_FIELD,
            ],
        ),
        (P::ArtifactMember, &[c::ARTIFACT_TARGET_FIELD]),
        (P::VideoArtifactMember, &[c::VIDEO_MUX_FIELD]),
        (
            P::ImageSequenceArtifactMember,
            &[
                c::IMAGE_SEQUENCE_NUMBERING_FIELD,
                c::IMAGE_SEQUENCE_ENCODE_FIELD,
            ],
        ),
        (
            P::CaptionSidecarArtifactMember,
            &[
                c::CAPTION_SIDECAR_SOURCE_FIELD,
                c::CAPTION_SIDECAR_ENCODE_FIELD,
            ],
        ),
        (
            P::AudioStemArtifactMember,
            &[c::AUDIO_STEM_SOURCE_FIELD, c::AUDIO_STEM_ENCODE_FIELD],
        ),
        (
            P::ScopeArtifactMember,
            &[
                c::SCOPE_ANALYZE_FIELD,
                c::SCOPE_FRAME_FIELD,
                c::SCOPE_CANVAS_FIELD,
                c::SCOPE_ENCODE_FIELD,
            ],
        ),
        (
            P::AudioFileArtifactMember,
            &[c::AUDIO_FILE_SOURCE_FIELD, c::AUDIO_FILE_ENCODE_FIELD],
        ),
        (
            P::AnimatedImageArtifactMember,
            &[c::ANIMATED_IMAGE_ENCODE_FIELD],
        ),
        (
            P::StillImageArtifactMember,
            &[c::STILL_IMAGE_FRAME_FIELD, c::STILL_IMAGE_ENCODE_FIELD],
        ),
        (
            P::AdaptivePackageArtifactMember,
            &[c::ADAPTIVE_PACKAGE_FIELD],
        ),
        (
            P::VideoMuxMember,
            &[
                c::VIDEO_MUX_LAYOUT_FIELD,
                c::VIDEO_MUX_VIDEO_FIELD,
                c::VIDEO_MUX_AUDIO_FIELD,
                c::VIDEO_MUX_PASSES_FIELD,
                c::VIDEO_MUX_ACCELERATOR_FIELD,
            ],
        ),
        (
            P::VideoEncodingMember,
            &[
                c::VIDEO_PIXEL_FORMAT_FIELD,
                c::VIDEO_ALPHA_FIELD,
                c::VIDEO_COLOR_SPACE_FIELD,
                c::VIDEO_RATE_CONTROL_FIELD,
                c::VIDEO_GOP_FIELD,
                c::VIDEO_B_FRAMES_FIELD,
                c::VIDEO_PROFILE_FIELD,
                c::VIDEO_LEVEL_FIELD,
            ],
        ),
        (
            P::VideoAudioEncodingMember,
            &[
                c::VIDEO_AUDIO_SAMPLE_RATE_FIELD,
                c::VIDEO_AUDIO_CHANNEL_LAYOUT_FIELD,
            ],
        ),
        (P::CrfRateControlMember, &[c::CRF_VALUE_FIELD]),
        (P::AverageRateControlMember, &[c::AVERAGE_TARGET_FIELD]),
        (
            P::CappedRateControlMember,
            &[
                c::CAPPED_TARGET_FIELD,
                c::CAPPED_MAX_FIELD,
                c::CAPPED_BUFFER_FIELD,
            ],
        ),
        (
            P::ColorSpaceMember,
            &[
                c::COLOR_SPACE_PRIMARIES_FIELD,
                c::COLOR_SPACE_TRANSFER_FIELD,
                c::COLOR_SPACE_MATRIX_FIELD,
                c::COLOR_SPACE_RANGE_FIELD,
            ],
        ),
    ]
}
