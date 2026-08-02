use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use veac_ir::{EditBatch, EditOperation, EditOutcome, OperationId, ProjectEnvelope, StructureEdit};

use crate::{OtioError, OtioImportResult, OtioLossReport};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OtioEditProposal {
    pub source_format: String,
    pub loss_report: OtioLossReport,
    pub batch: EditBatch,
}

pub fn propose_import(
    project: &ProjectEnvelope,
    imported: &OtioImportResult,
    operation_id: OperationId,
    allow_lossy: bool,
) -> Result<OtioEditProposal, OtioError> {
    if !imported.loss_report.is_empty() && !allow_lossy {
        return Err(OtioError::Loss(imported.loss_report.clone()));
    }
    if project
        .project
        .sequences
        .iter()
        .any(|value| value.id == imported.sequence.id)
    {
        return Err(OtioError::Edit(
            "target project already contains the imported sequence ID".to_owned(),
        ));
    }
    let mut operations = Vec::new();
    for material in &imported.materials {
        match project
            .project
            .materials
            .iter()
            .find(|value| value.id == material.id)
        {
            Some(existing) if existing == material => {}
            Some(_) => {
                return Err(OtioError::Edit(format!(
                    "material {} conflicts with the target project",
                    material.id
                )))
            }
            None => operations.push(EditOperation::EditStructure {
                edit: StructureEdit::InsertMaterial {
                    material: Box::new(material.clone()),
                    before_id: None,
                    after_id: None,
                },
            }),
        }
    }
    for group in &imported.multicam_groups {
        match project
            .project
            .multicam_groups
            .iter()
            .find(|value| value.id == group.id)
        {
            Some(existing) if existing == group => {}
            Some(_) => {
                return Err(OtioError::Edit(format!(
                    "multicam group {} conflicts with the target project",
                    group.id
                )))
            }
            None => operations.push(EditOperation::EditStructure {
                edit: StructureEdit::InsertMulticamGroup {
                    group: Box::new(group.clone()),
                    before_id: None,
                    after_id: None,
                },
            }),
        }
    }
    operations.push(EditOperation::EditStructure {
        edit: StructureEdit::InsertSequence {
            sequence: Box::new(imported.sequence.clone()),
            relations: imported.relations.clone(),
            before_id: None,
            after_id: None,
        },
    });
    for annotation in &imported.annotations {
        match project
            .project
            .annotations
            .iter()
            .find(|value| value.id == annotation.id)
        {
            Some(existing) if existing == annotation => {}
            Some(_) => {
                return Err(OtioError::Edit(format!(
                    "annotation {} conflicts with the target project",
                    annotation.id
                )))
            }
            None => operations.push(EditOperation::InsertAnnotation {
                annotation: Box::new(annotation.clone()),
            }),
        }
    }
    let batch = EditBatch {
        operation_id,
        base_revision: project.project.revision,
        atomic: true,
        preconditions: vec![],
        operations,
    };
    ensure_applicable(project, &batch)?;
    Ok(OtioEditProposal {
        source_format: "OpenTimelineIO".to_owned(),
        loss_report: imported.loss_report.clone(),
        batch,
    })
}

pub fn canonical_otio_proposal_json(value: &OtioEditProposal) -> Result<String, OtioError> {
    serde_json_canonicalizer::to_string(value).map_err(Into::into)
}

pub fn otio_edit_proposal_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(OtioEditProposal))
}

fn ensure_applicable(project: &ProjectEnvelope, batch: &EditBatch) -> Result<(), OtioError> {
    match veac_ir::apply_edit_batch(project, batch) {
        EditOutcome::Applied { .. } | EditOutcome::NoChange { .. } => Ok(()),
        EditOutcome::Conflict { diagnostics, .. } | EditOutcome::Rejected { diagnostics, .. } => {
            let message = diagnostics
                .iter()
                .map(|item| format!("{}: {}", item.code, item.message))
                .collect::<Vec<_>>()
                .join("; ");
            Err(OtioError::Edit(message))
        }
    }
}
