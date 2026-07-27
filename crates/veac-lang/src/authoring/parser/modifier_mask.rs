use crate::authoring::{
    Identifier, MaskModifierDecl, MaskShapeDecl, SemanticBlock, SemanticEntry, SemanticValue,
    VectorDecl,
};

use super::{parameter, semantic, Parser};

pub(super) fn parse(
    parser: &mut Parser,
    id: Identifier,
    mut body: SemanticBlock,
    span: crate::authoring::Span,
) -> Option<MaskModifierDecl> {
    let shape_entry = semantic::required(parser, &mut body, "shape", "mask")?;
    let shape = shape(parser, &shape_entry)?;
    let position = optional_vector(parser, &mut body, "position")?;
    let scale = optional_vector(parser, &mut body, "scale")?;
    let rotation = optional_scalar(parser, &mut body, "rotation")?;
    let feather = optional_scalar(parser, &mut body, "feather")?;
    let expansion = optional_scalar(parser, &mut body, "expansion")?;
    let invert = match semantic::take(parser, &mut body, "invert") {
        Some(entry) => Some(semantic::boolean(parser, &entry, "mask invert")?),
        None => None,
    };
    semantic::finish(parser, body, "mask");
    Some(MaskModifierDecl {
        id,
        shape,
        position,
        scale,
        rotation,
        feather,
        expansion,
        invert,
        span,
    })
}

fn shape(parser: &mut Parser, entry: &SemanticEntry) -> Option<MaskShapeDecl> {
    let [SemanticValue::Identifier(kind)] = entry.values.as_slice() else {
        semantic::shape(parser, entry, "shape expects one kind".to_owned());
        return None;
    };
    match kind.value.as_str() {
        "linear" if entry.block.is_none() => Some(MaskShapeDecl::Linear),
        "mirror" if entry.block.is_none() => Some(MaskShapeDecl::Mirror),
        "circle" if entry.block.is_none() => Some(MaskShapeDecl::Circle),
        "rectangle" if entry.block.is_none() => Some(MaskShapeDecl::Rectangle),
        "rounded-rectangle" => rounded_rectangle(parser, entry),
        "ellipse" if entry.block.is_none() => Some(MaskShapeDecl::Ellipse),
        "polygon" => point_shape(parser, entry, "polygon"),
        "heart" if entry.block.is_none() => Some(MaskShapeDecl::Heart),
        "star" if entry.block.is_none() => Some(MaskShapeDecl::Star),
        "path" => point_shape(parser, entry, "path"),
        _ => {
            parser.error(
                "AUTHORING_UNKNOWN_MASK_SHAPE",
                format!("unknown mask shape '{}'", kind.value),
                kind.span,
            );
            None
        }
    }
}

fn rounded_rectangle(parser: &mut Parser, entry: &SemanticEntry) -> Option<MaskShapeDecl> {
    let Some(block) = entry.block.clone() else {
        semantic::shape(
            parser,
            entry,
            "rounded-rectangle shape requires a block".to_owned(),
        );
        return None;
    };
    let mut block = block;
    let radius_entry = semantic::required(parser, &mut block, "radius", "rounded rectangle")?;
    let radius = semantic::number(parser, &radius_entry, "rounded rectangle radius")?;
    semantic::finish(parser, block, "rounded rectangle");
    Some(MaskShapeDecl::RoundedRectangle {
        radius,
        span: entry.span,
    })
}

fn point_shape(parser: &mut Parser, entry: &SemanticEntry, kind: &str) -> Option<MaskShapeDecl> {
    let Some(block) = entry.block.clone() else {
        semantic::shape(parser, entry, format!("{kind} shape requires a block"));
        return None;
    };
    let points = block
        .entries
        .iter()
        .filter_map(|entry| point(parser, entry))
        .collect::<Vec<_>>();
    if points.len() < 3 {
        parser.error(
            "AUTHORING_MASK_PATH_POINTS",
            format!("mask {kind} requires at least three points"),
            block.span,
        );
    }
    Some(if kind == "polygon" {
        MaskShapeDecl::Polygon {
            points,
            span: entry.span,
        }
    } else {
        MaskShapeDecl::Path {
            points,
            span: entry.span,
        }
    })
}

fn point(parser: &mut Parser, entry: &SemanticEntry) -> Option<VectorDecl> {
    if entry.name.value != "point" || !entry.values.is_empty() {
        semantic::shape(
            parser,
            entry,
            "path accepts only point { x; y; }".to_owned(),
        );
        return None;
    }
    let mut block = entry.block.clone()?;
    let x_entry = semantic::required(parser, &mut block, "x", "mask point")?;
    let x = semantic::number(parser, &x_entry, "mask point x")?;
    let y_entry = semantic::required(parser, &mut block, "y", "mask point")?;
    let y = semantic::number(parser, &y_entry, "mask point y")?;
    semantic::finish(parser, block, "mask point");
    Some(VectorDecl {
        x,
        y,
        span: entry.span,
    })
}

fn optional_vector(
    parser: &mut Parser,
    body: &mut SemanticBlock,
    name: &'static str,
) -> Option<Option<crate::authoring::ParameterDecl<VectorDecl>>> {
    match semantic::take(parser, body, name) {
        Some(entry) => parameter::vector(parser, &entry).map(Some),
        None => Some(None),
    }
}

fn optional_scalar(
    parser: &mut Parser,
    body: &mut SemanticBlock,
    name: &'static str,
) -> Option<Option<crate::authoring::ParameterDecl<crate::authoring::NumberLiteral>>> {
    match semantic::take(parser, body, name) {
        Some(entry) => parameter::scalar(parser, &entry).map(Some),
        None => Some(None),
    }
}
