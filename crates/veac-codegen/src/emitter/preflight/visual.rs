use veac_plan::canonical::{Length, Placement, Shadow};
use veac_plan::ResolvedSequence;

use super::Check;

pub(super) fn validate(check: &mut Check, sequence: &ResolvedSequence) {
    for clip in sequence.tracks.iter().flat_map(|track| &track.clips) {
        let Some(visual) = &clip.visual else { continue };
        let placement = match visual.placement {
            Placement::Anchor { inset, .. } => {
                inset.x.is_finite() && inset.y.is_finite() && inset.x >= 0.0 && inset.y >= 0.0
            }
            Placement::Absolute { position } => {
                length(position.x, false) && length(position.y, false)
            }
        };
        let frame = visual.frame.is_none_or(|value| {
            length(value.width, true)
                && length(value.height, true)
                && frame_geometry(value, sequence)
        });
        let anchor = visual.transform.anchor;
        let anchor = anchor.x.is_finite()
            && anchor.y.is_finite()
            && (0.0..=1.0).contains(&anchor.x)
            && (0.0..=1.0).contains(&anchor.y);
        let card = visual.card.as_ref().is_none_or(|value| {
            value.corner_radius_pixels.is_finite()
                && value.corner_radius_pixels >= 0.0
                && value.shadow.as_ref().is_none_or(shadow)
        });
        if !(placement && frame && anchor && card) {
            check.push(
                "PLAN_VISUAL_INVALID",
                Some(clip.id.to_string()),
                "visual placement, frame, anchor, card, or shadow is invalid",
            );
        }
    }
}

fn frame_geometry(value: veac_plan::canonical::Frame, sequence: &ResolvedSequence) -> bool {
    let width = pixels(value.width, sequence.settings.width);
    let height = pixels(value.height, sequence.settings.height);
    width
        .zip(height)
        .is_some_and(|(width, height)| veac_plan::canonical::ffmpeg_dimensions_valid(width, height))
}

fn pixels(value: Length, extent: u32) -> Option<u32> {
    let pixels = match value.unit {
        veac_plan::canonical::LengthUnit::Pixels => value.value,
        veac_plan::canonical::LengthUnit::Normalized => value.value * f64::from(extent),
        veac_plan::canonical::LengthUnit::Percent => value.value * f64::from(extent) / 100.0,
    };
    (pixels.is_finite() && pixels > 0.0 && pixels <= f64::from(u32::MAX))
        .then(|| pixels.round().max(1.0) as u32)
}

fn length(value: Length, positive: bool) -> bool {
    value.value.is_finite() && (!positive || value.value > 0.0)
}

pub(super) fn shadow(value: &Shadow) -> bool {
    veac_plan::canonical::shadow_blur_valid(value.blur_pixels)
        && value.opacity.is_finite()
        && (0.0..=1.0).contains(&value.opacity)
        && value.offset.x.is_finite()
        && value.offset.y.is_finite()
}
