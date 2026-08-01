mod outputs;
mod structure;
mod timeline;
mod visual;

use veac_plan::ResolvedRenderPlan;

use super::Check;

pub(super) fn validate_structural(check: &mut Check, plan: &ResolvedRenderPlan) {
    structure::validate(check, plan);
    timeline::validate(check, plan);
    outputs::validate(check, plan);
}

pub(super) fn validate_visual(check: &mut Check, plan: &ResolvedRenderPlan) {
    visual::validate(check, plan);
}
