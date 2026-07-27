use crate::authoring::{ItemDecl, LayerDecl, LayerKind, SequenceDecl, SourceDecl};

use super::mapping::mapping;
use super::modifier::modifier;
use super::structure::structure;
use super::value::reference;
use super::writer::Writer;

pub(super) fn sequence(writer: &mut Writer, value: &SequenceDecl) {
    writer.block(format!("sequence {}", value.id.value), |writer| {
        for (index, layer) in value.layers.iter().enumerate() {
            if index > 0 {
                writer.blank();
            }
            format_layer(writer, layer);
        }
        for value in &value.structures {
            writer.blank();
            structure(writer, value);
        }
    });
}

fn format_layer(writer: &mut Writer, value: &LayerDecl) {
    writer.block(
        format!("layer {} {}", layer_kind(value.kind), value.id.value),
        |writer| {
            if let Some(order) = &value.order {
                writer.line(format!("order {};", order.raw));
            }
            if let Some(bus) = &value.route_bus {
                writer.line(format!("route bus {};", bus.value));
            }
            for item in &value.items {
                if value.order.is_some()
                    || value.route_bus.is_some()
                    || item.id.value != value.items[0].id.value
                {
                    writer.blank();
                }
                format_item(writer, item);
            }
        },
    );
}

fn format_item(writer: &mut Writer, value: &ItemDecl) {
    writer.block(format!("item {}", value.id.value), |writer| {
        source(writer, &value.source);
        writer.block("record", |writer| {
            writer.line(format!("at {};", value.record.at.raw));
            writer.line(format!("duration {};", value.record.duration.raw));
        });
        if let Some(value) = &value.mapping {
            mapping(writer, value);
        }
        if let Some(value) = &value.template_slot {
            super::template_slot::template_slot(writer, value);
        }
        if !value.modifiers.is_empty() {
            writer.block("modifiers", |writer| {
                for value in &value.modifiers {
                    modifier(writer, value);
                }
            });
        }
    });
}

fn source(writer: &mut Writer, value: &SourceDecl) {
    if let SourceDecl::Generated { generator, .. } = value {
        generator_source(writer, generator);
        return;
    }
    if let SourceDecl::Text { text, .. } = value {
        super::text::source(writer, text);
        return;
    }
    if let SourceDecl::Caption { caption, .. } = value {
        super::text::caption_source(writer, caption);
        return;
    }
    if matches!(value, SourceDecl::Multicam { .. }) {
        super::multicam::source(writer, value);
        return;
    }
    let line = match value {
        SourceDecl::Media { resource, .. } => format!("source media {};", reference(resource)),
        SourceDecl::Text { .. } => unreachable!(),
        SourceDecl::Caption { .. } => unreachable!(),
        SourceDecl::Generated { .. } => unreachable!(),
        SourceDecl::Sequence { sequence, .. } => {
            format!("source sequence {};", reference(sequence))
        }
        SourceDecl::Multicam { .. } => unreachable!(),
    };
    writer.line(line);
}

fn generator_source(writer: &mut Writer, value: &crate::authoring::GeneratorDecl) {
    super::generator::source(writer, value);
}

fn layer_kind(value: LayerKind) -> &'static str {
    match value {
        LayerKind::Video => "video",
        LayerKind::Audio => "audio",
        LayerKind::Visual => "visual",
        LayerKind::Caption => "caption",
    }
}
