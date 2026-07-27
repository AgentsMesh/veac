use crate::authoring::{ApplyDecl, ApplyItemTarget, ApplyScope, StructureDecl};

use super::modifier::apply_stage;
use super::relation::relation;
use super::writer::Writer;

pub(super) fn structure(writer: &mut Writer, value: &StructureDecl) {
    match value {
        StructureDecl::Relation(value) => relation(writer, value),
        StructureDecl::Apply(value) => apply(writer, value),
    }
}

fn apply(writer: &mut Writer, value: &ApplyDecl) {
    writer.block(format!("apply {}", value.id.value), |writer| {
        scope(writer, &value.scope);
        writer.block("record", |writer| {
            writer.line(format!("at {};", value.record.at.raw));
            writer.line(format!("duration {};", value.record.duration.raw));
        });
        writer.block("pipeline", |writer| {
            for value in &value.pipeline {
                apply_stage(writer, value);
            }
        });
        writer.block("mix", |writer| {
            if let Some(value) = &value.mix.opacity {
                super::parameter::scalar(writer, "opacity", value);
            }
            if let Some(value) = &value.mix.blend {
                writer.line(format!("blend {};", value.value));
            }
            for value in &value.mix.masks {
                super::modifier_mask::mask(writer, value);
            }
        });
    });
}

fn scope(writer: &mut Writer, value: &ApplyScope) {
    match value {
        ApplyScope::CompositeBand { from, through, .. } => {
            writer.block("scope composite-band", |writer| {
                writer.line(format!("from layer {};", from.value));
                writer.line(format!("through layer {};", through.value));
            });
        }
        ApplyScope::Layer { layer, .. } => writer.line(format!("scope layer {};", layer.value)),
        ApplyScope::Items { targets, .. } => writer.block("scope items", |writer| {
            for target in targets {
                let (kind, id) = match target {
                    ApplyItemTarget::Item(value) => ("item", value),
                    ApplyItemTarget::Group(value) => ("group", value),
                };
                writer.line(format!("{kind} {};", id.value));
            }
        }),
    }
}
