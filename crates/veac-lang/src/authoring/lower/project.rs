use std::collections::BTreeMap;

use crate::authoring::{Diagnostics, Document};
use veac_ir::{Project, ProjectEnvelope, SequenceSettings};

use super::{
    annotation, apply, context::Context, delivery, ids, material, multicam, relation, settings,
    timeline,
};

pub fn lower_document(document: &Document) -> Result<ProjectEnvelope, Diagnostics> {
    let declaration = &document.project;
    let settings = settings::lower(&declaration.settings, declaration.span)?;
    let mut ctx = Context::new(settings.timescale);
    ctx.canvas_width = f64::from(settings.width);
    ctx.canvas_height = f64::from(settings.height);
    ctx.audio_resources = declaration
        .resources
        .iter()
        .filter(|resource| {
            resource
                .streams
                .as_ref()
                .is_some_and(|streams| streams.audio != crate::authoring::StreamSelection::Disabled)
        })
        .map(|resource| resource.id.value.clone())
        .collect();
    let sequence_settings = SequenceSettings {
        width: settings.width,
        height: settings.height,
        frame_rate: settings.frame_rate,
        sample_rate: settings.sample_rate,
    };
    let materials = declaration
        .resources
        .iter()
        .filter_map(|value| material::lower(&mut ctx, value))
        .collect();
    let mut multicam_groups: Vec<_> = declaration
        .multicams
        .iter()
        .filter_map(|value| multicam::group(&mut ctx, value))
        .collect();
    multicam_groups.sort_by(|left, right| left.id.cmp(&right.id));
    ctx.audio_sequences = super::sequence_audio::derive(&ctx, &declaration.sequences);
    let mut sequences = Vec::new();
    let mut relations = Vec::new();
    for value in &declaration.sequences {
        if let Some(mut sequence) = timeline::sequence(&mut ctx, value, &sequence_settings) {
            let scoped_relations = relation::lower_sequence(&mut ctx, value, &sequence);
            apply::lower_sequence(&mut ctx, value, &mut sequence, &scoped_relations);
            relations.extend(scoped_relations);
            sequences.push(sequence);
        }
    }
    let render_configs = declaration
        .deliveries
        .iter()
        .filter_map(|value| delivery::lower(&mut ctx, value))
        .collect();
    let annotations = declaration
        .annotations
        .iter()
        .filter_map(|value| annotation::lower(&mut ctx, value, &declaration.id))
        .collect();
    let envelope = ProjectEnvelope::new(Project {
        id: ids::project(&mut ctx, &declaration.id).unwrap_or_else(fallback_project_id),
        revision: 0,
        timebase: settings.timescale,
        entry_sequence_id: ids::sequence(&mut ctx, &declaration.entry.id)
            .unwrap_or_else(fallback_sequence_id),
        render_configs,
        materials,
        multicam_groups,
        annotations,
        relations,
        sequences,
        applied_operations: Vec::new(),
        metadata: BTreeMap::new(),
    });
    finish(ctx, envelope, declaration.span)
}

fn finish(
    mut ctx: Context,
    envelope: ProjectEnvelope,
    span: crate::authoring::Span,
) -> Result<ProjectEnvelope, Diagnostics> {
    if ctx.is_valid() {
        if let Err(error) = veac_ir::validate(&envelope) {
            for diagnostic in error.into_diagnostics() {
                ctx.error(
                    "AUTHORING_LOWER_IR_VALIDATION",
                    format!(
                        "{} at {}: {}",
                        diagnostic.code, diagnostic.pointer, diagnostic.message
                    ),
                    span,
                );
            }
        }
    }
    if ctx.is_valid() {
        Ok(envelope)
    } else {
        Err(ctx.diagnostics())
    }
}

fn fallback_project_id() -> veac_ir::ProjectId {
    veac_ir::ProjectId::new("prj_invalid").expect("static fallback id")
}

fn fallback_sequence_id() -> veac_ir::SequenceId {
    veac_ir::SequenceId::new("seq_invalid").expect("static fallback id")
}
