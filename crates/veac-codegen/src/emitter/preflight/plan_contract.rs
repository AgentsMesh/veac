use veac_plan::ResolvedRenderPlan;

use super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    let Err(errors) = veac_plan::validate_render_plan(plan) else {
        return;
    };
    for diagnostic in errors.into_diagnostics() {
        check.plan_contract(format!(
            "{} at {}: {}",
            diagnostic.code, diagnostic.pointer, diagnostic.message
        ));
    }
}
