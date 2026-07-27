use crate::*;

use crate::edit::operation_error;

#[cfg(test)]
mod tests;

pub(super) fn number<'a>(
    clip: &'a mut Clip,
    target: &NumberCurveTarget,
) -> Result<(&'a mut Animatable<f64>, Option<EffectId>), Diagnostic> {
    match target {
        NumberCurveTarget::VisualOpacity => Ok((&mut visual(clip)?.opacity, None)),
        NumberCurveTarget::VisualRotationDegrees => {
            Ok((&mut visual(clip)?.transform.rotation_degrees, None))
        }
        NumberCurveTarget::MaskRotationDegrees { mask_index } => {
            Ok((&mut mask(clip, *mask_index)?.rotation_degrees, None))
        }
        NumberCurveTarget::MaskFeatherPixels { mask_index } => {
            Ok((&mut mask(clip, *mask_index)?.feather_pixels, None))
        }
        NumberCurveTarget::MaskExpansionPixels { mask_index } => {
            Ok((&mut mask(clip, *mask_index)?.expansion_pixels, None))
        }
        NumberCurveTarget::AudioGain => Ok((&mut audio(clip)?.gain, None)),
        NumberCurveTarget::AudioPan => Ok((&mut audio(clip)?.pan, None)),
        NumberCurveTarget::TextReveal => Ok((&mut text_animation(clip)?.reveal, None)),
        NumberCurveTarget::TextOpacity => Ok((&mut text_animation(clip)?.opacity, None)),
        NumberCurveTarget::TextRotationDegrees => {
            Ok((&mut text_animation(clip)?.transform.rotation_degrees, None))
        }
        NumberCurveTarget::EffectParameter { effect_id, name } => {
            let effect = clip
                .effects
                .iter_mut()
                .find(|value| value.id == *effect_id)
                .ok_or_else(|| operation_error(effect_id.as_str(), "effect does not exist"))?;
            let parameter = effect.parameters.get_mut(name).ok_or_else(|| {
                operation_error(effect_id.as_str(), "effect parameter does not exist")
            })?;
            if let ParameterValue::Number { value } = parameter {
                *parameter = ParameterValue::NumberCurve {
                    value: Animatable::constant(*value),
                };
            }
            match parameter {
                ParameterValue::NumberCurve { value } => Ok((value, Some(effect_id.clone()))),
                _ => Err(operation_error(
                    effect_id.as_str(),
                    "effect parameter is not numeric",
                )),
            }
        }
    }
}

pub(super) fn point(
    clip: &mut Clip,
    target: PointCurveTarget,
) -> Result<&mut Animatable<Point>, Diagnostic> {
    match target {
        PointCurveTarget::VisualPosition => Ok(&mut visual(clip)?.transform.position),
        PointCurveTarget::TextPositionOffset => {
            Ok(&mut text_animation(clip)?.transform.position_offset)
        }
    }
}

pub(super) fn vec2<'a>(
    clip: &'a mut Clip,
    target: &Vec2CurveTarget,
) -> Result<&'a mut Animatable<Vec2>, Diagnostic> {
    match target {
        Vec2CurveTarget::VisualScale => Ok(&mut visual(clip)?.transform.scale),
        Vec2CurveTarget::TextScale => Ok(&mut text_animation(clip)?.transform.scale),
        Vec2CurveTarget::MaskPosition { mask_index } => Ok(&mut mask(clip, *mask_index)?.position),
        Vec2CurveTarget::MaskScale { mask_index } => Ok(&mut mask(clip, *mask_index)?.scale),
    }
}

pub(super) fn rect(
    clip: &mut Clip,
    target: RectCurveTarget,
) -> Result<&mut Animatable<Rect>, Diagnostic> {
    let clip_id = clip.id.clone();
    match target {
        RectCurveTarget::VisualCrop => visual(clip)?
            .transform
            .crop
            .as_mut()
            .ok_or_else(|| operation_error(clip_id.as_str(), "clip has no crop viewport")),
    }
}

fn mask(clip: &mut Clip, index: u32) -> Result<&mut Mask, Diagnostic> {
    let clip_id = clip.id.clone();
    let index = usize::try_from(index)
        .map_err(|_| operation_error(clip_id.as_str(), "mask index cannot be represented"))?;
    visual(clip)?
        .masks
        .get_mut(index)
        .ok_or_else(|| operation_error(clip_id.as_str(), "mask does not exist"))
}

fn visual(clip: &mut Clip) -> Result<&mut VisualProperties, Diagnostic> {
    clip.visual
        .as_mut()
        .ok_or_else(|| operation_error(clip.id.as_str(), "clip has no visual properties"))
}

fn audio(clip: &mut Clip) -> Result<&mut AudioProperties, Diagnostic> {
    clip.audio
        .as_mut()
        .ok_or_else(|| operation_error(clip.id.as_str(), "clip has no audio properties"))
}

fn text_animation(clip: &mut Clip) -> Result<&mut TextAnimation, Diagnostic> {
    let style = match &mut clip.source {
        ClipSource::Text { style, .. } | ClipSource::Caption { style, .. } => style,
        _ => return Err(operation_error(clip.id.as_str(), "clip is not text")),
    };
    style
        .animation
        .as_mut()
        .ok_or_else(|| operation_error(clip.id.as_str(), "text has no animation"))
}
