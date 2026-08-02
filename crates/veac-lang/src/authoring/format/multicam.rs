use crate::authoring::{MulticamDecl, MulticamSyncBasisDecl, SourceDecl};

use super::value::reference;
use super::writer::Writer;

pub(super) fn declaration(writer: &mut Writer, value: &MulticamDecl) {
    writer.block(format!("multicam {}", value.id.value), |writer| {
        writer.block(format!("sync {}", sync_basis(value.sync.basis)), |writer| {
            writer.line(format!("reference {};", reference(&value.sync.reference)));
        });
        for angle in &value.angles {
            writer.blank();
            writer.block(format!("angle {}", angle.id.value), |writer| {
                writer.line(format!("source {};", reference(&angle.resource)));
                writer.line(format!("source-offset {};", angle.source_offset.raw));
            });
        }
    });
}

pub(super) fn source(writer: &mut Writer, value: &SourceDecl) {
    let SourceDecl::Multicam {
        group, switches, ..
    } = value
    else {
        return;
    };
    writer.block(format!("source multicam {}", reference(group)), |writer| {
        for (index, switch) in switches.iter().enumerate() {
            if index > 0 {
                writer.blank();
            }
            writer.block(format!("switch {}", reference(&switch.angle)), |writer| {
                writer.line(format!("at {};", switch.at.raw));
                writer.line(format!("duration {};", switch.duration.raw));
            });
        }
    });
}

fn sync_basis(value: MulticamSyncBasisDecl) -> &'static str {
    match value {
        MulticamSyncBasisDecl::Timecode => "timecode",
        MulticamSyncBasisDecl::Audio => "audio",
        MulticamSyncBasisDecl::Manual => "manual",
    }
}
