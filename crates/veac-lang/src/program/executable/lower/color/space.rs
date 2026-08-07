use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{ColorMatrix, ColorPrimaries, ColorRange, ColorSpace, ColorTransfer};

use super::{malformed, ExecutableLowerError};
use crate::program::executable::lower::value;

pub(in crate::program::executable::lower) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<ColorSpace, ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::ColorSpace)?;
    if operands.len() != 4 {
        return Err(malformed());
    }
    Ok(ColorSpace {
        primaries: primaries(graph, &operands[0])?,
        transfer: transfer(graph, &operands[1])?,
        matrix: matrix(graph, &operands[2])?,
        range: range(graph, &operands[3])?,
    })
}

fn primaries(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<ColorPrimaries, ExecutableLowerError> {
    Ok(match empty(graph, source)? {
        Op::PrimariesBt709 => ColorPrimaries::Bt709,
        Op::PrimariesBt470m => ColorPrimaries::Bt470M,
        Op::PrimariesBt470bg => ColorPrimaries::Bt470Bg,
        Op::PrimariesSmpte170m => ColorPrimaries::Smpte170M,
        Op::PrimariesSmpte240m => ColorPrimaries::Smpte240M,
        Op::PrimariesFilm => ColorPrimaries::Film,
        Op::PrimariesBt2020 => ColorPrimaries::Bt2020,
        Op::PrimariesSmpte428 => ColorPrimaries::Smpte428,
        Op::PrimariesSmpte431 => ColorPrimaries::Smpte431,
        Op::PrimariesSmpte432 => ColorPrimaries::Smpte432,
        _ => return Err(malformed()),
    })
}

fn transfer(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<ColorTransfer, ExecutableLowerError> {
    Ok(match empty(graph, source)? {
        Op::TransferBt709 => ColorTransfer::Bt709,
        Op::TransferGamma22 => ColorTransfer::Gamma22,
        Op::TransferGamma28 => ColorTransfer::Gamma28,
        Op::TransferSmpte170m => ColorTransfer::Smpte170M,
        Op::TransferSmpte240m => ColorTransfer::Smpte240M,
        Op::TransferLinear => ColorTransfer::Linear,
        Op::TransferSrgb => ColorTransfer::Srgb,
        Op::TransferBt202010 => ColorTransfer::Bt2020_10,
        Op::TransferBt202012 => ColorTransfer::Bt2020_12,
        Op::TransferSmpte2084 => ColorTransfer::Smpte2084,
        Op::TransferAribStdB67 => ColorTransfer::AribStdB67,
        _ => return Err(malformed()),
    })
}

fn matrix(graph: &FrozenDomainGraph, source: &Value) -> Result<ColorMatrix, ExecutableLowerError> {
    Ok(match empty(graph, source)? {
        Op::MatrixRgb => ColorMatrix::Rgb,
        Op::MatrixBt709 => ColorMatrix::Bt709,
        Op::MatrixFcc => ColorMatrix::Fcc,
        Op::MatrixBt470bg => ColorMatrix::Bt470Bg,
        Op::MatrixSmpte170m => ColorMatrix::Smpte170M,
        Op::MatrixSmpte240m => ColorMatrix::Smpte240M,
        Op::MatrixYcgco => ColorMatrix::Ycgco,
        Op::MatrixBt2020Ncl => ColorMatrix::Bt2020Ncl,
        _ => return Err(malformed()),
    })
}

fn range(graph: &FrozenDomainGraph, source: &Value) -> Result<ColorRange, ExecutableLowerError> {
    Ok(match empty(graph, source)? {
        Op::RangeLimited => ColorRange::Limited,
        Op::RangeFull => ColorRange::Full,
        _ => return Err(malformed()),
    })
}

fn empty(graph: &FrozenDomainGraph, source: &Value) -> Result<Op, ExecutableLowerError> {
    let (operation, operands) = value::description(graph, source)?;
    operands
        .is_empty()
        .then_some(operation)
        .ok_or_else(malformed)
}
