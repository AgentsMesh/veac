use crate::authoring::{AudioProcessorDecl, AudioProcessorKindDecl, CompressorDecl, GateDecl};

use super::writer::Writer;

pub(super) fn processor(writer: &mut Writer, value: &AudioProcessorDecl) {
    let id = &value.id.value;
    match &value.kind {
        AudioProcessorKindDecl::ParametricEq { bands } => {
            writer.block(format!("processor eq {id}"), |writer| {
                for band in bands {
                    writer.block(format!("band {}", band.id.value), |writer| {
                        number(writer, "frequency", &band.frequency);
                        number(writer, "gain", &band.gain);
                        number(writer, "q", &band.q);
                    });
                }
            });
        }
        AudioProcessorKindDecl::HighPass(value) => filter(writer, "high-pass", id, value),
        AudioProcessorKindDecl::LowPass(value) => filter(writer, "low-pass", id, value),
        AudioProcessorKindDecl::Compressor(value) => compressor(writer, id, value),
        AudioProcessorKindDecl::Limiter(value) => {
            writer.block(format!("processor limiter {id}"), |writer| {
                number(writer, "ceiling", &value.ceiling);
                number(writer, "attack", &value.attack);
                number(writer, "release", &value.release);
            });
        }
        AudioProcessorKindDecl::Gate(value) => gate(writer, id, value),
        AudioProcessorKindDecl::Loudness(value) => {
            writer.block(format!("processor loudness {id}"), |writer| {
                number(writer, "integrated", &value.integrated);
                number(writer, "true-peak", &value.true_peak);
                number(writer, "range", &value.range);
            });
        }
    }
}

fn filter(writer: &mut Writer, name: &str, id: &str, value: &crate::authoring::FilterDecl) {
    writer.block(format!("processor {name} {id}"), |writer| {
        number(writer, "frequency", &value.frequency);
        number(writer, "q", &value.q);
        number(writer, "poles", &value.poles);
    });
}

fn compressor(writer: &mut Writer, id: &str, value: &CompressorDecl) {
    writer.block(format!("processor compressor {id}"), |writer| {
        number(writer, "threshold", &value.threshold);
        number(writer, "ratio", &value.ratio);
        number(writer, "attack", &value.attack);
        number(writer, "release", &value.release);
        number(writer, "knee", &value.knee);
        number(writer, "makeup-gain", &value.makeup_gain);
        number(writer, "mix", &value.mix);
    });
}

fn gate(writer: &mut Writer, id: &str, value: &GateDecl) {
    writer.block(format!("processor gate {id}"), |writer| {
        number(writer, "threshold", &value.threshold);
        number(writer, "ratio", &value.ratio);
        number(writer, "attack", &value.attack);
        number(writer, "release", &value.release);
        number(writer, "range", &value.range);
    });
}

fn number(writer: &mut Writer, name: &str, value: &crate::authoring::NumberLiteral) {
    writer.line(format!("{name} {};", value.raw));
}
