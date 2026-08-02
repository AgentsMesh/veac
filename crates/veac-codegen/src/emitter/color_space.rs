use veac_plan::canonical::{ColorMatrix, ColorPrimaries, ColorRange, ColorSpace, ColorTransfer};

use super::EmitContext;

pub(super) fn convert(
    context: &mut EmitContext<'_>,
    input: String,
    from: ColorSpace,
    to: ColorSpace,
) -> String {
    if from == to {
        return tag(context, input, to);
    }
    let (color_format, alpha_format) = if to.matrix == ColorMatrix::Rgb {
        ("gbrpf32le", "grayf32le")
    } else {
        ("yuv444p16le", "gray16le")
    };
    let filter = format!(
        "zscale=matrixin={}:primariesin={}:transferin={}:rangein={}:matrix={}:primaries={}:transfer={}:range={},format={color_format}",
        matrix(from.matrix),
        primaries(from.primaries),
        zscale_transfer(from.transfer),
        range(from.range),
        matrix(to.matrix),
        primaries(to.primaries),
        zscale_transfer(to.transfer),
        range(to.range),
    );
    let (color, alpha) = context.graph.split(&input, "colorspacesplitv");
    let color = context.graph.filter(&[&color], filter, "colorspacev");
    let alpha = context.graph.filter(
        &[&alpha],
        format!("format=gbrapf32le,alphaextract,format={alpha_format}"),
        "colorspacealphav",
    );
    let output_format = if to.matrix == ColorMatrix::Rgb {
        "gbrapf32le"
    } else {
        "yuva444p16le"
    };
    super::alpha_merge::apply(context, &color, &alpha, output_format, "colorspacemergev")
}

fn zscale_transfer(value: ColorTransfer) -> &'static str {
    match value {
        ColorTransfer::Gamma22 => "bt470m",
        ColorTransfer::Gamma28 => "bt470bg",
        _ => transfer(value),
    }
}

pub(super) fn tag(context: &mut EmitContext<'_>, input: String, value: ColorSpace) -> String {
    context.graph.filter(
        &[&input],
        format!(
            "setparams=range={}:color_primaries={}:color_trc={}:colorspace={}",
            set_range(value.range),
            primaries(value.primaries),
            transfer(value.transfer),
            matrix(value.matrix)
        ),
        "colortagv",
    )
}

pub(super) fn primaries(value: ColorPrimaries) -> &'static str {
    match value {
        ColorPrimaries::Bt709 => "bt709",
        ColorPrimaries::Bt470M => "bt470m",
        ColorPrimaries::Bt470Bg => "bt470bg",
        ColorPrimaries::Smpte170M => "smpte170m",
        ColorPrimaries::Smpte240M => "smpte240m",
        ColorPrimaries::Film => "film",
        ColorPrimaries::Bt2020 => "bt2020",
        ColorPrimaries::Smpte428 => "smpte428",
        ColorPrimaries::Smpte431 => "smpte431",
        ColorPrimaries::Smpte432 => "smpte432",
    }
}

pub(super) fn transfer(value: ColorTransfer) -> &'static str {
    match value {
        ColorTransfer::Bt709 => "bt709",
        ColorTransfer::Gamma22 => "gamma22",
        ColorTransfer::Gamma28 => "gamma28",
        ColorTransfer::Smpte170M => "smpte170m",
        ColorTransfer::Smpte240M => "smpte240m",
        ColorTransfer::Linear => "linear",
        ColorTransfer::Srgb => "iec61966-2-1",
        ColorTransfer::Bt2020_10 => "bt2020-10",
        ColorTransfer::Bt2020_12 => "bt2020-12",
        ColorTransfer::Smpte2084 => "smpte2084",
        ColorTransfer::AribStdB67 => "arib-std-b67",
    }
}

pub(super) fn matrix(value: ColorMatrix) -> &'static str {
    match value {
        ColorMatrix::Rgb => "gbr",
        ColorMatrix::Bt709 => "bt709",
        ColorMatrix::Fcc => "fcc",
        ColorMatrix::Bt470Bg => "bt470bg",
        ColorMatrix::Smpte170M => "smpte170m",
        ColorMatrix::Smpte240M => "smpte240m",
        ColorMatrix::Ycgco => "ycgco",
        ColorMatrix::Bt2020Ncl => "bt2020nc",
    }
}

pub(super) fn range(value: ColorRange) -> &'static str {
    match value {
        ColorRange::Limited => "tv",
        ColorRange::Full => "pc",
    }
}

fn set_range(value: ColorRange) -> &'static str {
    match value {
        ColorRange::Limited => "limited",
        ColorRange::Full => "full",
    }
}
