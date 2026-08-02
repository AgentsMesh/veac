use std::collections::BTreeMap;

use crate::program::diagnostic::Diagnostic;
use crate::program::expression::ValueLookup;
use crate::program::model::{ComponentCatalog, InstanceDecl, ResolvedComponent};
use crate::program::provenance::ProvenanceMap;

use super::super::{hygiene, inject, slot, text};
use super::{expand_one, provenance, Caller, State};

#[allow(clippy::too_many_arguments)]
pub(super) fn expand(
    caller: &Caller<'_>,
    catalog: &ComponentCatalog,
    instance: &InstanceDecl,
    path: &hygiene::InstancePath,
    component: &ResolvedComponent,
    values: &dyn ValueLookup,
    fills: &BTreeMap<String, String>,
    output_id: &str,
    provenance: &mut ProvenanceMap,
    registry: &mut hygiene::Registry,
    sequences: &mut inject::Sequences<'_>,
    state: &mut State,
) -> Result<(), Diagnostic> {
    let nested_caller = Caller {
        path: &component.source_path,
        source: &component.source,
        values,
        presets: component.captured.presets.as_ref(),
        components: &component.captured.components,
        forwarded_slots: fills,
        hygiene_path: Some(path.clone()),
    };
    for child in &component.declaration.instances {
        let child_path = path.child(&child.id);
        expand_one(
            &nested_caller,
            catalog,
            child,
            &child_path,
            provenance,
            registry,
            sequences,
            state,
        )?;
    }
    let retained = state.retained_frame_bytes();
    let limit = sequences.remaining(retained)?;
    let body = expanded_body(component, values, fills, path, registry, limit)?;
    provenance::locals(component, path, output_id, provenance)?;
    provenance::instance(
        provenance,
        caller.path,
        instance.span,
        output_id.to_owned(),
        instance.component.clone(),
    )?;
    sequences.push(output_id, body, retained)
}

fn expanded_body(
    component: &ResolvedComponent,
    values: &dyn ValueLookup,
    fills: &BTreeMap<String, String>,
    path: &hygiene::InstancePath,
    registry: &mut hygiene::Registry,
    limit: usize,
) -> Result<String, Diagnostic> {
    let span = component.declaration.body.content_span;
    let raw = &component.source[span.start..span.end];
    let expanded = text::expand_with_limit(
        &component.source_path,
        raw,
        component.captured.presets.as_ref(),
        values,
        0,
        limit,
    )?;
    let hygienic =
        hygiene::expand_with_limit(&component.source_path, expanded, path, registry, limit)?;
    slot::inject_with_limit(&component.source_path, hygienic, fills, limit)
}
