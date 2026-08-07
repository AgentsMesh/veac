use veac_plan::EffectiveVisualProperties;

use crate::emitter::process_owner::ProcessOwner;
use crate::unit_tests::emitter_tests::support::{fixture, resolved};

pub(crate) fn overlay_position(
    visual: &EffectiveVisualProperties,
    local_clock: &str,
    pivot_x: f64,
    pivot_y: f64,
) -> (String, String) {
    let plan = resolved(&fixture());
    let clip = &plan.sequences[0].tracks[0].clips[0];
    super::position(
        visual,
        &plan,
        ProcessOwner::clip(clip),
        local_clock,
        pivot_x,
        pivot_y,
        super::CoordinateSpace {
            width: "W",
            height: "H",
            source_width: "w",
            source_height: "h",
        },
    )
}
