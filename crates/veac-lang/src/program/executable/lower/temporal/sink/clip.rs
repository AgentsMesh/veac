use crate::program::executable::{
    ClipTemporalProperty as ClipProperty, ExecutableTemporalSink, MaskTemporalProperty,
    TextTemporalProperty,
};
use veac_ir::{Animatable, Clip, ClipSource, Effect, EffectParameter, Project, TemporalBindingId};

use super::super::{error, ExecutableLowerError};

pub(super) fn attach(
    project: &mut Project,
    target: &ExecutableTemporalSink,
    binding_id: TemporalBindingId,
) -> Result<(), ExecutableLowerError> {
    let item_id = target.item_id().expect("clip sink has item owner");
    let clip = project
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
        .flat_map(|track| &mut track.clips)
        .find(|clip| &clip.id == item_id)
        .ok_or_else(missing)?;
    match target {
        ExecutableTemporalSink::Clip { property, .. } => basic(clip, *property, binding_id),
        ExecutableTemporalSink::ClipMask {
            mask_index,
            property,
            ..
        } => {
            let mask = visual(clip)?
                .masks
                .get_mut(*mask_index as usize)
                .ok_or_else(optional)?;
            mask_value(mask, *property, binding_id);
            Ok(())
        }
        ExecutableTemporalSink::ClipText { property, .. } => text(clip, *property, binding_id),
        ExecutableTemporalSink::ClipEffect {
            effect_id,
            parameter,
            ..
        } => {
            let effect = clip
                .effects
                .iter_mut()
                .find(|effect| &effect.id == effect_id)
                .ok_or_else(missing)?;
            parameter_value(&mut effect.effect, *parameter, binding_id)
        }
        _ => unreachable!("apply sinks dispatch separately"),
    }
}

fn basic(
    clip: &mut Clip,
    property: ClipProperty,
    id: TemporalBindingId,
) -> Result<(), ExecutableLowerError> {
    match property {
        ClipProperty::VisualPosition => visual(clip)?.transform.position = binding(id),
        ClipProperty::VisualScale => visual(clip)?.transform.scale = binding(id),
        ClipProperty::VisualRotation => visual(clip)?.transform.rotation_degrees = binding(id),
        ClipProperty::VisualCrop => {
            *visual(clip)?.transform.crop.as_mut().ok_or_else(optional)? = binding(id)
        }
        ClipProperty::VisualOpacity => visual(clip)?.opacity = binding(id),
        ClipProperty::AudioGain => audio(clip)?.gain = binding(id),
        ClipProperty::AudioPan => audio(clip)?.pan = binding(id),
    }
    Ok(())
}

fn text(
    clip: &mut Clip,
    property: TextTemporalProperty,
    id: TemporalBindingId,
) -> Result<(), ExecutableLowerError> {
    let style = match &mut clip.source {
        ClipSource::Text { style, .. } | ClipSource::Caption { style, .. } => style,
        _ => {
            return Err(owner(
                "the temporal text target is not a text or caption clip",
            ))
        }
    };
    let animation = style.animation.as_mut().ok_or_else(optional)?;
    match property {
        TextTemporalProperty::Position => animation.transform.position_offset = binding(id),
        TextTemporalProperty::Scale => animation.transform.scale = binding(id),
        TextTemporalProperty::Rotation => animation.transform.rotation_degrees = binding(id),
        TextTemporalProperty::Reveal => animation.reveal = binding(id),
        TextTemporalProperty::HighlightProgress => {
            animation.highlight.as_mut().ok_or_else(optional)?.progress = binding(id)
        }
        TextTemporalProperty::Opacity => animation.opacity = binding(id),
    }
    Ok(())
}

pub(super) fn mask_value(
    mask: &mut veac_ir::Mask,
    property: MaskTemporalProperty,
    id: TemporalBindingId,
) {
    match property {
        MaskTemporalProperty::Position => mask.position = binding(id),
        MaskTemporalProperty::Scale => mask.scale = binding(id),
        MaskTemporalProperty::Rotation => mask.rotation_degrees = binding(id),
        MaskTemporalProperty::Feather => mask.feather_pixels = binding(id),
        MaskTemporalProperty::Expansion => mask.expansion_pixels = binding(id),
    }
}

pub(super) fn parameter_value(
    effect: &mut Effect,
    parameter: EffectParameter,
    id: TemporalBindingId,
) -> Result<(), ExecutableLowerError> {
    let value = effect.curve_mut(parameter).ok_or_else(|| {
        owner("the effect parameter is not an animatable number leaf of the selected effect")
    })?;
    *value = binding(id);
    Ok(())
}

fn visual(clip: &mut Clip) -> Result<&mut veac_ir::VisualProperties, ExecutableLowerError> {
    clip.visual.as_mut().ok_or_else(|| {
        error(
            "EXECUTABLE_TEMPORAL_VISUAL_SINK",
            "the requested temporal visual leaf has no static visual owner",
        )
    })
}

fn audio(clip: &mut Clip) -> Result<&mut veac_ir::AudioProperties, ExecutableLowerError> {
    clip.audio.as_mut().ok_or_else(|| {
        error(
            "EXECUTABLE_TEMPORAL_AUDIO_SINK",
            "the requested temporal audio leaf has no static audio owner",
        )
    })
}

fn binding<T>(binding_id: TemporalBindingId) -> Animatable<T> {
    Animatable::Binding { binding_id }
}

fn missing() -> ExecutableLowerError {
    error(
        "EXECUTABLE_TEMPORAL_SINK_MISSING",
        "the requested static temporal leaf does not exist",
    )
}

fn optional() -> ExecutableLowerError {
    error(
        "EXECUTABLE_TEMPORAL_OPTIONAL_SINK",
        "a temporal declaration cannot create an absent optional leaf",
    )
}

fn owner(message: &'static str) -> ExecutableLowerError {
    error("EXECUTABLE_TEMPORAL_SINK_OWNER", message)
}
