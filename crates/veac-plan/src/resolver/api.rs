use veac_ir::{ProjectEnvelope, RenderConfigId};

use crate::{ResolutionDiagnostic, ResolutionErrorKind, ResolutionErrors, ResolvedRenderPlan};

use super::{
    error::{canonical_errors, internal_hash_error, normalize_diagnostics},
    selection::select_configs,
    PlanResolver,
};

/// Resolve all configs, or one selected config, without performing environment I/O.
///
/// Returned plans are sorted by render-config ID. Disabled clips and clips on non-solo tracks
/// under an active solo are omitted before material requirements are evaluated.
pub fn resolve(
    envelope: &ProjectEnvelope,
    render_config_id: Option<&RenderConfigId>,
) -> Result<Vec<ResolvedRenderPlan>, ResolutionErrors> {
    if let Err(errors) = veac_ir::validate(envelope) {
        return Err(canonical_errors(errors));
    }
    let configs = select_configs(envelope, render_config_id)?;
    let semantic = veac_ir::semantic_hash(envelope).map_err(internal_hash_error)?;
    let snapshot = veac_ir::snapshot_hash(envelope).map_err(internal_hash_error)?;
    let mut plans = Vec::with_capacity(configs.len());
    let mut diagnostics = Vec::new();
    for config in configs {
        let resolver = PlanResolver::new(envelope, config, &semantic, &snapshot);
        let (plan, mut errors) = resolver.build();
        diagnostics.append(&mut errors);
        plans.extend(plan);
    }
    normalize_diagnostics(&mut diagnostics);
    if diagnostics.is_empty() {
        Ok(plans)
    } else {
        Err(ResolutionErrors::new(diagnostics))
    }
}

/// Resolve exactly one render config.
pub fn resolve_one(
    envelope: &ProjectEnvelope,
    render_config_id: &RenderConfigId,
) -> Result<ResolvedRenderPlan, ResolutionErrors> {
    let mut plans = resolve(envelope, Some(render_config_id))?;
    plans.pop().ok_or_else(|| {
        ResolutionErrors::new(vec![ResolutionDiagnostic::new(
            ResolutionErrorKind::InternalInvariant,
            "EMPTY_PLAN_RESULT",
            Some(render_config_id.to_string()),
            "/project/render_configs",
            "selected render config did not produce a plan",
        )])
    })
}
