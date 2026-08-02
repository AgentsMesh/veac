use crate::authoring::{MappingDecl, MappingKey, SourceOutOfRangeDecl};

use super::writer::Writer;

pub(super) fn mapping(writer: &mut Writer, value: &MappingDecl) {
    match value {
        MappingDecl::Linear {
            from, to, outside, ..
        } => writer.block("mapping linear", |writer| {
            writer.line(format!("from {};", from.raw));
            writer.line(format!("to {};", to.raw));
            outside_policy(writer, *outside);
        }),
        MappingDecl::Freeze { source, .. } => writer.block("mapping freeze", |writer| {
            writer.line(format!("source {};", source.raw));
        }),
        MappingDecl::Curve { keys, outside, .. } => writer.block("mapping curve", |writer| {
            for key in keys {
                mapping_key(writer, key);
            }
            outside_policy(writer, *outside);
        }),
    }
}

fn outside_policy(writer: &mut Writer, value: SourceOutOfRangeDecl) {
    let value = match value {
        SourceOutOfRangeDecl::Strict => "strict",
        SourceOutOfRangeDecl::HoldFirst => "hold-first",
        SourceOutOfRangeDecl::HoldLast => "hold-last",
        SourceOutOfRangeDecl::HoldBoth => "hold-both",
    };
    writer.line(format!("outside {value};"));
}

fn mapping_key(writer: &mut Writer, value: &MappingKey) {
    writer.block(format!("key {}", value.id.value), |writer| {
        writer.line(format!("at {};", value.at.raw));
        writer.line(format!("source {};", value.source.raw));
        super::interpolation::write(writer, &value.interpolation);
    });
}
