#![allow(dead_code)]

mod material;
mod project;

pub use material::*;
pub use project::*;

use veac_ir::*;
use veac_template::{MediaBinding, TemplateFillRequest, TextBinding, TEMPLATE_FILL_SCHEMA_ID};

pub fn request(project: &ProjectEnvelope, material: Material) -> TemplateFillRequest {
    TemplateFillRequest {
        schema: TEMPLATE_FILL_SCHEMA_ID.to_owned(),
        schema_version: 1,
        operation_id: OperationId::new("op_fill_template").unwrap(),
        base_revision: project.project.revision,
        media_bindings: vec![MediaBinding {
            clip_id: ItemId::new("itm_slot").unwrap(),
            material,
        }],
        text_bindings: vec![],
    }
}

pub fn text_binding(text: &str) -> TextBinding {
    TextBinding {
        clip_id: ItemId::new("itm_title").unwrap(),
        text: text.to_owned(),
    }
}

pub fn apply(project: &ProjectEnvelope, batch: &EditBatch) -> ProjectEnvelope {
    match apply_edit_batch(project, batch) {
        EditOutcome::Applied { project, .. } => project,
        other => panic!("expected applied edit, got {other:?}"),
    }
}

pub fn clip<'a>(project: &'a ProjectEnvelope, id: &str) -> &'a Clip {
    project
        .project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
        .find(|clip| clip.id.as_str() == id)
        .unwrap()
}

pub fn linear(clip: &Clip) -> (RationalTime, Rational) {
    let SourceTimeMap::Linear {
        source_start, rate, ..
    } = clip
        .source_mapping
        .as_ref()
        .expect("source mapping")
        .time_map
    else {
        panic!("linear mapping");
    };
    (source_start, rate)
}

pub fn error_kind(
    project: &ProjectEnvelope,
    request: &TemplateFillRequest,
) -> veac_template::TemplateErrorKind {
    veac_template::propose_template_fill(project, request)
        .unwrap_err()
        .kind
}
