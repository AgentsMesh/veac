use super::{controls, Diagnostic, Parser};
use crate::program::model::TemporalProperty;

pub(super) fn parse(parser: &mut Parser<'_>) -> Result<TemporalProperty, Diagnostic> {
    let choices = [
        (
            controls::TEMPORAL_VISUAL_POSITION,
            TemporalProperty::VisualPosition,
        ),
        (
            controls::TEMPORAL_VISUAL_SCALE,
            TemporalProperty::VisualScale,
        ),
        (
            controls::TEMPORAL_VISUAL_ROTATION,
            TemporalProperty::VisualRotation,
        ),
        (controls::TEMPORAL_VISUAL_CROP, TemporalProperty::VisualCrop),
        (
            controls::TEMPORAL_VISUAL_OPACITY,
            TemporalProperty::VisualOpacity,
        ),
        (controls::TEMPORAL_AUDIO_GAIN, TemporalProperty::AudioGain),
        (controls::TEMPORAL_AUDIO_PAN, TemporalProperty::AudioPan),
        (
            controls::TEMPORAL_MASK_POSITION,
            TemporalProperty::MaskPosition,
        ),
        (controls::TEMPORAL_MASK_SCALE, TemporalProperty::MaskScale),
        (
            controls::TEMPORAL_MASK_ROTATION,
            TemporalProperty::MaskRotation,
        ),
        (
            controls::TEMPORAL_MASK_FEATHER,
            TemporalProperty::MaskFeather,
        ),
        (
            controls::TEMPORAL_MASK_EXPANSION,
            TemporalProperty::MaskExpansion,
        ),
        (
            controls::TEMPORAL_TEXT_POSITION,
            TemporalProperty::TextPosition,
        ),
        (controls::TEMPORAL_TEXT_SCALE, TemporalProperty::TextScale),
        (
            controls::TEMPORAL_TEXT_ROTATION,
            TemporalProperty::TextRotation,
        ),
        (controls::TEMPORAL_TEXT_REVEAL, TemporalProperty::TextReveal),
        (
            controls::TEMPORAL_TEXT_HIGHLIGHT_PROGRESS,
            TemporalProperty::TextHighlightProgress,
        ),
        (
            controls::TEMPORAL_TEXT_OPACITY,
            TemporalProperty::TextOpacity,
        ),
        (
            controls::TEMPORAL_EFFECT_PARAMETER,
            TemporalProperty::EffectParameter,
        ),
        (
            controls::TEMPORAL_APPLY_OPACITY,
            TemporalProperty::ApplyOpacity,
        ),
    ];
    choices
        .into_iter()
        .find_map(|(control, property)| parser.at_control(control).then_some(property))
        .inspect(|_| {
            parser.advance();
        })
        .ok_or_else(|| {
            parser.error(
                "PROGRAM_TEMPORAL_PROPERTY",
                "expected one closed temporal property",
                parser.current().span,
            )
        })
}
