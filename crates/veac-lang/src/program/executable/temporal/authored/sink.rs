use crate::program::executable::lower::id;
use crate::program::model::{TemporalProperty as Property, TemporalTarget as Target};

use super::super::{
    ClipTemporalProperty as Clip, ExecutableTemporalSink as Sink, MaskTemporalProperty as Mask,
    TextTemporalProperty as Text,
};

pub(in crate::program::executable::temporal) fn lower(
    target: &Target,
    property: Property,
) -> Result<Sink, &'static str> {
    match (target, property) {
        (Target::Clip(path), property) => Ok(Sink::clip(
            id::item(&path.segments()),
            clip_property(property).ok_or_else(mismatch)?,
        )),
        (Target::Text(path), property) => Ok(Sink::ClipText {
            item_id: id::item(&path.segments()),
            property: text_property(property).ok_or_else(mismatch)?,
        }),
        (Target::ClipMask { clip, mask_index }, property) => Ok(Sink::ClipMask {
            item_id: id::item(&clip.segments()),
            mask_index: *mask_index,
            property: mask_property(property).ok_or_else(mismatch)?,
        }),
        (
            Target::ClipEffect {
                clip,
                effect,
                parameter,
            },
            Property::EffectParameter,
        ) => {
            let mut path = clip.segments().to_vec();
            path.push(effect);
            Ok(Sink::ClipEffect {
                item_id: id::item(&clip.segments()),
                effect_id: id::effect(&path),
                parameter: effect_parameter(parameter)?,
            })
        }
        (Target::Apply(path), Property::ApplyOpacity) => Ok(Sink::ApplyOpacity {
            sequence_id: id::sequence(&path.sequence_segments()),
            apply_id: id::apply(&path.segments()),
        }),
        (Target::ApplyMask { apply, mask_index }, property) => Ok(Sink::ApplyMask {
            sequence_id: id::sequence(&apply.sequence_segments()),
            apply_id: id::apply(&apply.segments()),
            mask_index: *mask_index,
            property: mask_property(property).ok_or_else(mismatch)?,
        }),
        (
            Target::ApplyEffect {
                apply,
                stage,
                effect,
                parameter,
            },
            Property::EffectParameter,
        ) => apply_effect(apply, stage, effect, parameter),
        _ => Err(mismatch()),
    }
}

pub(super) fn clip_sequence(target: &Target) -> Result<veac_ir::SequenceId, &'static str> {
    let path = target.item().ok_or_else(mismatch)?;
    Ok(super::super::super::lower::id::sequence(
        &path.sequence_segments(),
    ))
}

fn apply_effect(
    apply: &crate::program::model::TemporalApplyPath,
    stage: &str,
    effect: &str,
    parameter: &str,
) -> Result<Sink, &'static str> {
    let mut stage_path = apply.segments().to_vec();
    stage_path.push(stage);
    let mut effect_path = stage_path.clone();
    effect_path.push(effect);
    Ok(Sink::ApplyEffect {
        sequence_id: id::sequence(&apply.sequence_segments()),
        apply_id: id::apply(&apply.segments()),
        stage_id: id::apply_stage(&stage_path),
        effect_id: id::effect(&effect_path),
        parameter: effect_parameter(parameter)?,
    })
}

fn clip_property(value: Property) -> Option<Clip> {
    Some(match value {
        Property::VisualPosition => Clip::VisualPosition,
        Property::VisualScale => Clip::VisualScale,
        Property::VisualRotation => Clip::VisualRotation,
        Property::VisualCrop => Clip::VisualCrop,
        Property::VisualOpacity => Clip::VisualOpacity,
        Property::AudioGain => Clip::AudioGain,
        Property::AudioPan => Clip::AudioPan,
        _ => return None,
    })
}

fn mask_property(value: Property) -> Option<Mask> {
    Some(match value {
        Property::MaskPosition => Mask::Position,
        Property::MaskScale => Mask::Scale,
        Property::MaskRotation => Mask::Rotation,
        Property::MaskFeather => Mask::Feather,
        Property::MaskExpansion => Mask::Expansion,
        _ => return None,
    })
}

fn text_property(value: Property) -> Option<Text> {
    Some(match value {
        Property::TextPosition => Text::Position,
        Property::TextScale => Text::Scale,
        Property::TextRotation => Text::Rotation,
        Property::TextReveal => Text::Reveal,
        Property::TextHighlightProgress => Text::HighlightProgress,
        Property::TextOpacity => Text::Opacity,
        _ => return None,
    })
}

fn effect_parameter(value: &str) -> Result<veac_ir::EffectParameter, &'static str> {
    veac_ir::EffectParameter::from_name(value).ok_or_else(invalid_parameter)
}

const fn mismatch() -> &'static str {
    "temporal property is not valid for the selected typed target kind"
}

const fn invalid_parameter() -> &'static str {
    "effect selector is not a closed animatable effect parameter"
}

#[cfg(test)]
mod tests;
