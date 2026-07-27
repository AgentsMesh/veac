use crate::authoring::ColorModifierDecl;
use veac_ir::{ColorMatrix, ColorPrimaries, ColorRange, ColorTransfer};

use super::color_stage::stage;
use super::writer::Writer;

pub(super) fn color(writer: &mut Writer, value: &ColorModifierDecl) {
    write(writer, format!("color {}", value.id.value), value);
}

pub(super) fn apply_stage(writer: &mut Writer, value: &ColorModifierDecl) {
    write(writer, format!("stage color {}", value.id.value), value);
}

fn write(writer: &mut Writer, header: String, value: &ColorModifierDecl) {
    writer.block(header, |writer| {
        space(writer, "input-space", &value.input.value);
        space(writer, "working-space", &value.working.value);
        space(writer, "output-space", &value.output.value);
        for value in &value.stages {
            stage(writer, value);
        }
    });
}

fn space(writer: &mut Writer, name: &str, value: &veac_ir::ColorSpace) {
    writer.block(name, |writer| {
        writer.line(format!("primaries {};", primaries(value.primaries)));
        writer.line(format!("transfer {};", transfer(value.transfer)));
        writer.line(format!("matrix {};", matrix(value.matrix)));
        writer.line(format!("range {};", range(value.range)));
    });
}

fn primaries(value: ColorPrimaries) -> &'static str {
    match value {
        ColorPrimaries::Bt709 => "bt709",
        ColorPrimaries::Bt470M => "bt470-m",
        ColorPrimaries::Bt470Bg => "bt470-bg",
        ColorPrimaries::Smpte170M => "smpte170-m",
        ColorPrimaries::Smpte240M => "smpte240-m",
        ColorPrimaries::Film => "film",
        ColorPrimaries::Bt2020 => "bt2020",
        ColorPrimaries::Smpte428 => "smpte428",
        ColorPrimaries::Smpte431 => "smpte431",
        ColorPrimaries::Smpte432 => "smpte432",
    }
}

fn transfer(value: ColorTransfer) -> &'static str {
    match value {
        ColorTransfer::Bt709 => "bt709",
        ColorTransfer::Gamma22 => "gamma22",
        ColorTransfer::Gamma28 => "gamma28",
        ColorTransfer::Smpte170M => "smpte170-m",
        ColorTransfer::Smpte240M => "smpte240-m",
        ColorTransfer::Linear => "linear",
        ColorTransfer::Srgb => "srgb",
        ColorTransfer::Bt2020_10 => "bt2020-10",
        ColorTransfer::Bt2020_12 => "bt2020-12",
        ColorTransfer::Smpte2084 => "smpte2084",
        ColorTransfer::AribStdB67 => "arib-std-b67",
    }
}

fn matrix(value: ColorMatrix) -> &'static str {
    match value {
        ColorMatrix::Rgb => "rgb",
        ColorMatrix::Bt709 => "bt709",
        ColorMatrix::Fcc => "fcc",
        ColorMatrix::Bt470Bg => "bt470-bg",
        ColorMatrix::Smpte170M => "smpte170-m",
        ColorMatrix::Smpte240M => "smpte240-m",
        ColorMatrix::Ycgco => "ycgco",
        ColorMatrix::Bt2020Ncl => "bt2020-ncl",
    }
}

fn range(value: ColorRange) -> &'static str {
    match value {
        ColorRange::Full => "full",
        ColorRange::Limited => "limited",
    }
}
