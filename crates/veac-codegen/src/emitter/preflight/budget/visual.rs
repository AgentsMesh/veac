use veac_plan::canonical::{
    placement_intermediate_extent, visual_intermediate_pixels, Animatable,
    MAX_VISUAL_INTERMEDIATE_PIXELS,
};
use veac_plan::ResolvedRenderPlan;

use super::super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    for sequence in &plan.sequences {
        for clip in sequence.tracks.iter().flat_map(|track| &track.clips) {
            let Some(visual) = &clip.visual else { continue };
            let extent =
                super::super::super::visual_extent::placement_extent(plan, sequence, clip, visual);
            let placement = extent
                .filter(|_| {
                    matches!(visual.transform.position, Animatable::Constant { .. })
                        && super::super::super::visual_extent::requires_canvas_placement(
                            clip, visual,
                        )
                })
                .and_then(|extent| {
                    placement_intermediate_extent(
                        (sequence.settings.width, sequence.settings.height),
                        extent,
                    )
                });
            let shadow_blur = visual
                .card
                .as_ref()
                .and_then(|card| card.shadow.as_ref())
                .map(|shadow| shadow.blur_pixels);
            let exceeds = extent
                .and_then(|extent| {
                    visual_intermediate_pixels(
                        extent,
                        placement,
                        (sequence.settings.width, sequence.settings.height),
                        shadow_blur,
                    )
                })
                .is_none_or(|pixels| pixels > MAX_VISUAL_INTERMEDIATE_PIXELS);
            if exceeds {
                check.push(
                    "PLAN_BUDGET_VISUAL_INTERMEDIATE_PIXELS",
                    Some(clip.id.to_string()),
                    "visual transform, placement, or shadow exceeds the intermediate-frame memory budget",
                );
            }
        }
    }
}
