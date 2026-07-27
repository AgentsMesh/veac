use crate::authoring::{AudioProcessorDecl, CompressorDecl, GateDecl};

use super::writer::Writer;

pub(super) fn processor(writer: &mut Writer, value: &AudioProcessorDecl) {
    match value {
        AudioProcessorDecl::ParametricEq { bands, .. } => writer.block("processor eq", |writer| {
            for band in bands {
                writer.block("band", |writer| {
                    number(writer, "frequency", &band.frequency);
                    number(writer, "gain", &band.gain);
                    number(writer, "q", &band.q);
                });
            }
        }),
        AudioProcessorDecl::HighPass(value) => filter(writer, "high-pass", value),
        AudioProcessorDecl::LowPass(value) => filter(writer, "low-pass", value),
        AudioProcessorDecl::Compressor(value) => compressor(writer, value),
        AudioProcessorDecl::Limiter(value) => writer.block("processor limiter", |writer| {
            number(writer, "ceiling", &value.ceiling);
            number(writer, "attack", &value.attack);
            number(writer, "release", &value.release);
        }),
        AudioProcessorDecl::Gate(value) => gate(writer, value),
        AudioProcessorDecl::Loudness(value) => writer.block("processor loudness", |writer| {
            number(writer, "integrated", &value.integrated);
            number(writer, "true-peak", &value.true_peak);
            number(writer, "range", &value.range);
        }),
    }
}

fn filter(writer: &mut Writer, name: &str, value: &crate::authoring::FilterDecl) {
    writer.block(format!("processor {name}"), |writer| {
        number(writer, "frequency", &value.frequency);
        number(writer, "q", &value.q);
        number(writer, "poles", &value.poles);
    });
}

fn compressor(writer: &mut Writer, value: &CompressorDecl) {
    writer.block("processor compressor", |writer| {
        number(writer, "threshold", &value.threshold);
        number(writer, "ratio", &value.ratio);
        number(writer, "attack", &value.attack);
        number(writer, "release", &value.release);
        number(writer, "knee", &value.knee);
        number(writer, "makeup-gain", &value.makeup_gain);
        number(writer, "mix", &value.mix);
    });
}

fn gate(writer: &mut Writer, value: &GateDecl) {
    writer.block("processor gate", |writer| {
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
