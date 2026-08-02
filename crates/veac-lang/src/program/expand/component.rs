mod body;
mod context;
mod provenance;
mod state;

use std::collections::BTreeMap;

pub(super) use context::Caller;
pub(super) use state::State;

use super::{budget, hygiene, inject, parameters, slot};
use crate::program::diagnostic::Diagnostic;
use crate::program::model::{ComponentCatalog, InstanceDecl, ResolvedComponent, SurfaceFile};
use crate::program::provenance::ProvenanceMap;

#[allow(clippy::too_many_arguments)]
pub(super) fn expand(
    file: &SurfaceFile,
    scope: super::scope::View<'_>,
    catalog: &ComponentCatalog,
    instance: &InstanceDecl,
    provenance: &mut ProvenanceMap,
    registry: &mut hygiene::Registry,
    sequences: &mut inject::Sequences<'_>,
    state: &mut State,
) -> Result<(), Diagnostic> {
    let forwarded = BTreeMap::new();
    let caller = Caller::entry(file, scope, &forwarded);
    let path = hygiene::InstancePath::root(&instance.id);
    expand_one(
        &caller, catalog, instance, &path, provenance, registry, sequences, state,
    )
}

#[allow(clippy::too_many_arguments)]
fn expand_one(
    caller: &Caller<'_>,
    catalog: &ComponentCatalog,
    instance: &InstanceDecl,
    path: &hygiene::InstancePath,
    provenance: &mut ProvenanceMap,
    registry: &mut hygiene::Registry,
    sequences: &mut inject::Sequences<'_>,
    state: &mut State,
) -> Result<(), Diagnostic> {
    let key = caller.components.get(&instance.component).ok_or_else(|| {
        Diagnostic::new(
            "PROGRAM_COMPONENT_NOT_FOUND",
            caller.path,
            format!("component `{}` was not found", instance.component),
            instance.span,
        )
    })?;
    let component = catalog.get(key).ok_or_else(|| {
        Diagnostic::new(
            "PROGRAM_COMPONENT_NOT_FOUND",
            caller.path,
            format!(
                "component `{}` has no resolved definition",
                instance.component
            ),
            instance.span,
        )
    })?;
    state.enter(key, caller.path, instance.span)?;
    let result = expand_entered(
        caller, catalog, instance, path, component, provenance, registry, sequences, state,
    );
    state.leave();
    result
}

#[allow(clippy::too_many_arguments)]
fn expand_entered(
    caller: &Caller<'_>,
    catalog: &ComponentCatalog,
    instance: &InstanceDecl,
    path: &hygiene::InstancePath,
    component: &ResolvedComponent,
    provenance: &mut ProvenanceMap,
    registry: &mut hygiene::Registry,
    sequences: &mut inject::Sequences<'_>,
    state: &mut State,
) -> Result<(), Diagnostic> {
    parameters::validate_bindings(caller.path, instance, &component.interface)?;
    let values = parameters::bind(
        caller.path,
        caller.values,
        instance,
        component,
        state.remaining_frame_bytes(),
    )?;
    state.charge_parameter_work(
        caller.path,
        instance.span,
        component.interface.parameter_count(),
    )?;
    let output_id = registry.reserve(caller.path, path, instance.span)?;
    let parameter_bytes = values.retained_bytes();
    state.retain_frame(caller.path, parameter_bytes)?;
    let result = (|| {
        let fills = slot::expand_caller_fills(
            caller,
            &component.declaration.slots,
            &component.interface,
            &instance.fills,
            caller.hygiene_path.as_ref(),
            registry,
            state.remaining_frame_bytes(),
        )?;
        let fill_bytes = fills.values().try_fold(0usize, |total, fill| {
            budget::checked_source_add(caller.path, total, fill.len())
        })?;
        state.retain_frame(caller.path, fill_bytes)?;
        let result = body::expand(
            caller, catalog, instance, path, component, &values, &fills, &output_id, provenance,
            registry, sequences, state,
        );
        state.release_frame(fill_bytes);
        result
    })();
    state.release_frame(parameter_bytes);
    result
}
