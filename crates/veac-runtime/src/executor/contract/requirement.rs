use std::collections::{BTreeMap, BTreeSet};

use veac_codegen::emitter::BackendPhase;

use super::super::model::RuntimeBundle;
use crate::RuntimeError;

pub(super) fn validate(
    bundle: &RuntimeBundle,
    phases: &BTreeMap<String, Vec<BackendPhase>>,
) -> Result<(), RuntimeError> {
    let mut seen = BTreeSet::new();
    for requirement in &bundle.requirements {
        let kind = requirement.kind().as_str();
        let deliverable_id = requirement.deliverable_id();
        let name = requirement.name();
        if name.is_empty()
            || !phases.contains_key(&deliverable_id.to_string())
            || !seen.insert((kind, deliverable_id.to_string(), name.to_owned()))
        {
            return super::invalid("backend requirement is empty, duplicate, or orphaned");
        }
    }
    Ok(())
}
