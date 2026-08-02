use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::model::ResolvedComponent;
use crate::program::provenance::{Origin, ProvenanceMap};

use super::super::{hygiene, origin};

pub(super) fn locals(
    component: &ResolvedComponent,
    path: &hygiene::InstancePath,
    instance_id: &str,
    provenance: &mut ProvenanceMap,
) -> Result<(), Diagnostic> {
    for (local, span) in origin::local_declarations(
        &component.source_path,
        &component.source,
        component.declaration.body.content_span,
    )? {
        insert(
            provenance,
            path.child(&local).render(&component.source_path, span)?,
            Origin {
                path: component.source_path.clone(),
                span,
                instance_id: Some(instance_id.to_owned()),
                definition: Some(component.declaration.name.clone()),
            },
        )?;
    }
    Ok(())
}

pub(super) fn instance(
    provenance: &mut ProvenanceMap,
    path: &str,
    span: Span,
    id: String,
    definition: String,
) -> Result<(), Diagnostic> {
    insert(
        provenance,
        id.clone(),
        Origin {
            path: path.to_owned(),
            span,
            instance_id: Some(id),
            definition: Some(definition),
        },
    )
}

fn insert(provenance: &mut ProvenanceMap, id: String, origin: Origin) -> Result<(), Diagnostic> {
    if provenance.try_insert(id.clone(), origin.clone()) {
        return Ok(());
    }
    Err(Diagnostic::new(
        "PROGRAM_HYGIENIC_ID_COLLISION",
        &origin.path,
        format!("expanded ID `{id}` has more than one declaration origin"),
        origin.span,
    ))
}
