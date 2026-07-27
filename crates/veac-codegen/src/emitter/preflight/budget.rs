mod outputs;
mod structure;
mod timeline;
mod visual;

use veac_plan::ResolvedRenderPlan;

use super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    structure::validate(check, plan);
    timeline::validate(check, plan);
    outputs::validate(check, plan);
    visual::validate(check, plan);
}
