use std::collections::BTreeSet;

use veac_ir::{
    apply_edit_batch, validate, EditBatch, EditOperation, EditOutcome, Precondition,
    ProjectEnvelope, StructureEdit, VisualProperty,
};

use crate::{
    bindings, inventory, media, request, TemplateError, TemplateErrorKind, TemplateFillRequest,
};

pub fn propose_template_fill(
    project: &ProjectEnvelope,
    request: &TemplateFillRequest,
) -> Result<EditBatch, TemplateError> {
    validate(project).map_err(|error| {
        TemplateError::new(
            TemplateErrorKind::InvalidProject,
            format!("template project is invalid: {error}"),
        )
    })?;
    request::validate_header(request)?;
    if request.base_revision != project.project.revision {
        return Err(TemplateError::new(
            TemplateErrorKind::RevisionMismatch,
            "template fill request uses a stale project revision",
        ));
    }
    let inventory = inventory::collect(project);
    if inventory.slots.is_empty() && inventory.texts.is_empty() {
        return Err(TemplateError::new(
            TemplateErrorKind::NoTemplateTargets,
            "project has no replaceable media or editable text targets",
        ));
    }
    ensure_unlocked(&inventory)?;
    let bindings = bindings::resolve(&inventory, request)?;
    let mut operations = Vec::new();
    for slot in &inventory.slots {
        let material = bindings.media[&slot.clip_id];
        let plan = media::plan(slot, material, project.project.timebase)?;
        operations.push(EditOperation::EditStructure {
            edit: StructureEdit::SetMaterial {
                material_id: slot.material_id.clone(),
                material: Box::new(material.clone()),
            },
        });
        operations.push(EditOperation::SetSourceMapping {
            clip_id: slot.clip_id.clone(),
            source_mapping: plan.mapping,
        });
        operations.push(EditOperation::SetVisualProperty {
            clip_id: slot.clip_id.clone(),
            property: VisualProperty::Crop(Some(veac_ir::Animatable::constant(plan.crop))),
        });
        operations.push(clear(slot.clip_id.clone()));
    }
    for text in &inventory.texts {
        if let Some(value) = bindings.texts.get(&text.clip_id) {
            operations.push(EditOperation::SetText {
                clip_id: text.clip_id.clone(),
                text: (*value).to_owned(),
            });
        }
        operations.push(clear(text.clip_id.clone()));
    }
    let batch = EditBatch {
        operation_id: request.operation_id.clone(),
        base_revision: request.base_revision,
        atomic: true,
        preconditions: preconditions(&inventory),
        operations,
    };
    dry_run(project, &batch)?;
    Ok(batch)
}

fn ensure_unlocked(inventory: &inventory::Inventory) -> Result<(), TemplateError> {
    let locked = inventory
        .slots
        .iter()
        .map(|slot| (&slot.clip_id, slot.track_locked))
        .chain(
            inventory
                .texts
                .iter()
                .map(|text| (&text.clip_id, text.track_locked)),
        )
        .find(|(_, locked)| *locked);
    if let Some((id, _)) = locked {
        return Err(TemplateError::clip(
            TemplateErrorKind::LockedTrack,
            id,
            "template target belongs to a locked track",
        ));
    }
    Ok(())
}

fn preconditions(inventory: &inventory::Inventory) -> Vec<Precondition> {
    let mut result = Vec::new();
    let mut tracks = BTreeSet::new();
    for (clip_id, track_id, source) in inventory
        .slots
        .iter()
        .map(|slot| (&slot.clip_id, &slot.track_id, &slot.source))
        .chain(
            inventory
                .texts
                .iter()
                .map(|text| (&text.clip_id, &text.track_id, &text.source)),
        )
    {
        result.push(Precondition::ClipExists {
            clip_id: clip_id.clone(),
        });
        result.push(Precondition::ClipSourceEquals {
            clip_id: clip_id.clone(),
            source: Box::new(source.clone()),
        });
        if tracks.insert(track_id.clone()) {
            result.push(Precondition::TrackUnlocked {
                track_id: track_id.clone(),
            });
        }
    }
    result
}

fn clear(clip_id: veac_ir::ItemId) -> EditOperation {
    EditOperation::SetTemplateState {
        clip_id,
        replaceable: None,
        template_editable_text: false,
    }
}

fn dry_run(project: &ProjectEnvelope, batch: &EditBatch) -> Result<(), TemplateError> {
    match apply_edit_batch(project, batch) {
        EditOutcome::Applied { .. } | EditOutcome::NoChange { .. } => Ok(()),
        EditOutcome::Conflict { diagnostics, .. } | EditOutcome::Rejected { diagnostics, .. } => {
            let message = diagnostics
                .first()
                .map_or("template edit was rejected".to_owned(), |value| {
                    format!("template edit was rejected: {}", value.message)
                });
            Err(TemplateError::new(TemplateErrorKind::EditRejected, message))
        }
    }
}
