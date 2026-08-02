use std::collections::BTreeSet;

use veac_ir::{MaterialId, ProjectEnvelope, RenderConfigId};

use crate::ResolutionErrors;

use super::{error::canonical_errors, reachability::Reachability, selection::select_configs};

/// Return the exact material closure for one config without requiring identities or probe facts.
pub fn required_material_ids_one(
    envelope: &ProjectEnvelope,
    render_config_id: &RenderConfigId,
) -> Result<BTreeSet<MaterialId>, ResolutionErrors> {
    if let Err(errors) = veac_ir::validate(envelope) {
        return Err(canonical_errors(errors));
    }
    let config = select_configs(envelope, Some(render_config_id))?
        .into_iter()
        .next()
        .expect("selected config is guaranteed by select_configs");
    Ok(Reachability::analyze(envelope, config).material_ids())
}
