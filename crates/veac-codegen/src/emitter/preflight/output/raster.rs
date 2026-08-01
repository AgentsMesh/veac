use veac_plan::canonical::render_geometry_valid;
use veac_plan::ResolvedOutput;

use super::super::Check;

pub(super) fn validate(check: &mut Check, output: &ResolvedOutput) {
    let visual = output
        .deliverables
        .iter()
        .any(|value| value.kind.requires_raster());
    match (&output.raster, visual) {
        (Some(raster), true) => {
            if !render_geometry_valid(raster.width, raster.height, raster.frame_rate) {
                check.push(
                    "PLAN_RASTER_INVALID",
                    Some(output.id.to_string()),
                    "raster geometry exceeds the untrusted render budget",
                );
            }
        }
        (None, true) => check.push(
            "PLAN_RASTER_REQUIRED",
            Some(output.id.to_string()),
            "visual artifacts require delivery raster settings",
        ),
        (Some(_), false) => check.push(
            "PLAN_RASTER_UNUSED",
            Some(output.id.to_string()),
            "delivery raster must be omitted when no visual artifact consumes it",
        ),
        (None, false) => {}
    }
}
