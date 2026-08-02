use crate::authoring::{SemanticBlock, SemanticEntry, SemanticValue, Spanned};
use veac_ir::{ColorMatrix, ColorPrimaries, ColorRange, ColorSpace, ColorTransfer};

use super::{semantic, Parser};

pub(super) fn parse(parser: &mut Parser, entry: &SemanticEntry) -> Option<Spanned<ColorSpace>> {
    let mut block = semantic::nested(parser, entry, "color space")?;
    let primaries = field(parser, &mut block, "primaries", primaries)?;
    let transfer = field(parser, &mut block, "transfer", transfer)?;
    let matrix = field(parser, &mut block, "matrix", matrix)?;
    let range = field(parser, &mut block, "range", range)?;
    semantic::finish(parser, block, "color space");
    Some(Spanned {
        value: ColorSpace {
            primaries,
            transfer,
            matrix,
            range,
        },
        span: entry.span,
    })
}

fn field<T>(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
    parse: fn(&str) -> Option<T>,
) -> Option<T> {
    let entry = semantic::required(parser, block, name, "color space")?;
    let [SemanticValue::Identifier(value)] = entry.values.as_slice() else {
        semantic::shape(parser, &entry, format!("{name} expects one value"));
        return None;
    };
    parse(&value.value).or_else(|| {
        parser.error(
            "AUTHORING_UNKNOWN_ENUM_VALUE",
            format!("unknown {name} '{}'", value.value),
            value.span,
        );
        None
    })
}

fn primaries(value: &str) -> Option<ColorPrimaries> {
    Some(match value {
        "bt709" => ColorPrimaries::Bt709,
        "bt470-m" => ColorPrimaries::Bt470M,
        "bt470-bg" => ColorPrimaries::Bt470Bg,
        "smpte170-m" => ColorPrimaries::Smpte170M,
        "smpte240-m" => ColorPrimaries::Smpte240M,
        "film" => ColorPrimaries::Film,
        "bt2020" => ColorPrimaries::Bt2020,
        "smpte428" => ColorPrimaries::Smpte428,
        "smpte431" => ColorPrimaries::Smpte431,
        "smpte432" => ColorPrimaries::Smpte432,
        _ => return None,
    })
}

fn transfer(value: &str) -> Option<ColorTransfer> {
    Some(match value {
        "bt709" => ColorTransfer::Bt709,
        "gamma22" => ColorTransfer::Gamma22,
        "gamma28" => ColorTransfer::Gamma28,
        "smpte170-m" => ColorTransfer::Smpte170M,
        "smpte240-m" => ColorTransfer::Smpte240M,
        "linear" => ColorTransfer::Linear,
        "srgb" => ColorTransfer::Srgb,
        "bt2020-10" => ColorTransfer::Bt2020_10,
        "bt2020-12" => ColorTransfer::Bt2020_12,
        "smpte2084" => ColorTransfer::Smpte2084,
        "arib-std-b67" => ColorTransfer::AribStdB67,
        _ => return None,
    })
}

fn matrix(value: &str) -> Option<ColorMatrix> {
    Some(match value {
        "rgb" => ColorMatrix::Rgb,
        "bt709" => ColorMatrix::Bt709,
        "fcc" => ColorMatrix::Fcc,
        "bt470-bg" => ColorMatrix::Bt470Bg,
        "smpte170-m" => ColorMatrix::Smpte170M,
        "smpte240-m" => ColorMatrix::Smpte240M,
        "ycgco" => ColorMatrix::Ycgco,
        "bt2020-ncl" => ColorMatrix::Bt2020Ncl,
        _ => return None,
    })
}

fn range(value: &str) -> Option<ColorRange> {
    Some(match value {
        "full" => ColorRange::Full,
        "limited" => ColorRange::Limited,
        _ => return None,
    })
}
