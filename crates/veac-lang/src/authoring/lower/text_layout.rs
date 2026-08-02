use crate::authoring::TextLayoutDecl;
use veac_ir::{Length, LengthUnit, Point, TextLayout, TextPath};

use super::context::Context;
use super::value;

pub(super) fn lower(
    ctx: &mut Context,
    declaration: &TextLayoutDecl,
) -> Option<(TextLayout, Option<TextPath>)> {
    let default = TextLayout::default();
    let layout = TextLayout {
        box_width_pixels: optional_pixels(ctx, declaration.box_width.as_ref())?,
        box_height_pixels: optional_pixels(ctx, declaration.box_height.as_ref())?,
        wrap: declaration
            .wrap
            .as_ref()
            .map_or(default.wrap, |value| value.value),
        overflow: declaration
            .overflow
            .as_ref()
            .map_or(default.overflow, |value| value.value),
        horizontal_alignment: declaration
            .horizontal_alignment
            .as_ref()
            .map_or(default.horizontal_alignment, |value| value.value),
        vertical_alignment: declaration
            .vertical_alignment
            .as_ref()
            .map_or(default.vertical_alignment, |value| value.value),
        writing_mode: declaration
            .writing_mode
            .as_ref()
            .map_or(default.writing_mode, |value| value.value),
        orientation: declaration
            .orientation
            .as_ref()
            .map_or(default.orientation, |value| value.value),
    };
    let path = match &declaration.path {
        Some(path) => Some(TextPath {
            points: path
                .points
                .iter()
                .map(|point| {
                    Some(Point {
                        x: length(ctx, &point.x)?,
                        y: length(ctx, &point.y)?,
                    })
                })
                .collect::<Option<Vec<_>>>()?,
            start_offset: length(ctx, &path.start_offset)?,
            reverse: path.reverse.value,
            alignment: path.align.value,
        }),
        None => None,
    };
    Some((layout, path))
}

fn optional_pixels(
    ctx: &mut Context,
    value: Option<&crate::authoring::NumberLiteral>,
) -> Option<Option<f64>> {
    match value {
        Some(value) => Some(Some(super::value::scalar(ctx, value, "px")?)),
        None => Some(None),
    }
}

pub(super) fn length(ctx: &mut Context, value: &crate::authoring::NumberLiteral) -> Option<Length> {
    let (_, unit) = value::split(&value.raw);
    let number = value::number(ctx, value, &["px", "%"], "text length")?;
    Some(Length {
        value: number,
        unit: if unit == "%" {
            LengthUnit::Percent
        } else {
            LengthUnit::Pixels
        },
    })
}
