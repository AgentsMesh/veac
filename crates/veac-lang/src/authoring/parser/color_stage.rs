use crate::authoring::{
    ColorCurvePointDecl, ColorCurvesDecl, ColorWheelDecl, ColorWheelsDecl, RgbMatrixDecl,
    SemanticEntry, SemanticValue,
};

use super::{semantic, Parser};

pub(super) fn matrix(parser: &mut Parser, entry: &SemanticEntry) -> Option<RgbMatrixDecl> {
    let mut block = semantic::nested(parser, entry, "RGB matrix")?;
    let red_entry = semantic::required(parser, &mut block, "red", "matrix")?;
    let red = triple(parser, &red_entry)?;
    let green_entry = semantic::required(parser, &mut block, "green", "matrix")?;
    let green = triple(parser, &green_entry)?;
    let blue_entry = semantic::required(parser, &mut block, "blue", "matrix")?;
    let blue = triple(parser, &blue_entry)?;
    let offset_entry = semantic::required(parser, &mut block, "offset", "matrix")?;
    let offset = triple(parser, &offset_entry)?;
    semantic::finish(parser, block, "RGB matrix");
    Some(RgbMatrixDecl {
        rows: [red, green, blue],
        offset,
        span: entry.span,
    })
}

pub(super) fn curves(parser: &mut Parser, entry: &SemanticEntry) -> Option<ColorCurvesDecl> {
    let mut block = semantic::nested(parser, entry, "color curves")?;
    let interpolation_entry = semantic::required(parser, &mut block, "interpolation", "curves")?;
    let interpolation = semantic::word(parser, &interpolation_entry, "curve interpolation")?;
    let mut luma = None;
    let mut red = None;
    let mut green = None;
    let mut blue = None;
    for curve_entry in &block.entries {
        let (channel, points) = curve(parser, curve_entry)?;
        let slot = match channel.value.as_str() {
            "luma" => &mut luma,
            "red" => &mut red,
            "green" => &mut green,
            "blue" => &mut blue,
            _ => {
                parser.error(
                    "AUTHORING_UNKNOWN_COLOR_CHANNEL",
                    format!("unknown curve channel '{}'", channel.value),
                    channel.span,
                );
                continue;
            }
        };
        if slot.is_some() {
            parser.duplicate("curve channel", channel.span);
        } else {
            *slot = Some(points);
        }
    }
    Some(ColorCurvesDecl {
        interpolation,
        luma,
        red,
        green,
        blue,
        span: entry.span,
    })
}

pub(super) fn wheels(parser: &mut Parser, entry: &SemanticEntry) -> Option<ColorWheelsDecl> {
    let mut block = semantic::nested(parser, entry, "color wheels")?;
    let lift_entry = semantic::required(parser, &mut block, "lift", "wheels")?;
    let lift = wheel(parser, &lift_entry)?;
    let gamma_entry = semantic::required(parser, &mut block, "gamma", "wheels")?;
    let gamma = wheel(parser, &gamma_entry)?;
    let gain_entry = semantic::required(parser, &mut block, "gain", "wheels")?;
    let gain = wheel(parser, &gain_entry)?;
    semantic::finish(parser, block, "color wheels");
    Some(ColorWheelsDecl {
        lift,
        gamma,
        gain,
        span: entry.span,
    })
}

fn curve(
    parser: &mut Parser,
    entry: &SemanticEntry,
) -> Option<(crate::authoring::Identifier, Vec<ColorCurvePointDecl>)> {
    if entry.name.value != "curve" {
        semantic::shape(parser, entry, "curves accept only curve entries".to_owned());
        return None;
    }
    let [SemanticValue::Identifier(channel)] = entry.values.as_slice() else {
        semantic::shape(
            parser,
            entry,
            "curve expects one channel and block".to_owned(),
        );
        return None;
    };
    let block = entry.block.clone()?;
    let points = block
        .entries
        .iter()
        .filter_map(|value| point(parser, value))
        .collect();
    Some((channel.clone(), points))
}

fn triple(
    parser: &mut Parser,
    entry: &SemanticEntry,
) -> Option<[crate::authoring::NumberLiteral; 3]> {
    let [SemanticValue::Number(a), SemanticValue::Number(b), SemanticValue::Number(c)] =
        entry.values.as_slice()
    else {
        semantic::shape(parser, entry, "field expects three numbers".to_owned());
        return None;
    };
    Some([a.clone(), b.clone(), c.clone()])
}

fn wheel(parser: &mut Parser, entry: &SemanticEntry) -> Option<ColorWheelDecl> {
    let [red, green, blue] = triple(parser, entry)?;
    Some(ColorWheelDecl { red, green, blue })
}

fn point(parser: &mut Parser, entry: &SemanticEntry) -> Option<ColorCurvePointDecl> {
    if entry.name.value != "point" {
        semantic::shape(parser, entry, "curve accepts only point entries".to_owned());
        return None;
    }
    let [SemanticValue::Number(input), SemanticValue::Number(output)] = entry.values.as_slice()
    else {
        semantic::shape(parser, entry, "point expects input and output".to_owned());
        return None;
    };
    Some(ColorCurvePointDecl {
        input: input.clone(),
        output: output.clone(),
        span: entry.span,
    })
}
