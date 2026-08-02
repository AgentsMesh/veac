use crate::authoring::{
    CompositeModifierDecl, LayoutModifierDecl, PlacementDecl, TransformModifierDecl,
};
use veac_ir::{
    Anchor, BlendMode, FitMode, Frame, Length, LengthUnit, Placement, Point, Rect, Vec2,
    VisualProperties,
};

use super::animation;
use super::context::Context;
use super::value;

pub(super) fn layout(
    context: &mut Context,
    target: &mut VisualProperties,
    value: &LayoutModifierDecl,
) -> Option<()> {
    if let Some(value) = &value.placement {
        target.placement = match value {
            PlacementDecl::Anchor { anchor, inset, .. } => Placement::Anchor {
                anchor: anchor_value(context, anchor)?,
                inset: vector(context, inset, pixels)?,
            },
            PlacementDecl::Absolute { position, .. } => Placement::Absolute {
                position: point(context, position)?,
            },
        };
    }
    if let Some(value) = &value.frame {
        target.frame = Some(Frame {
            width: length(context, &value.width)?,
            height: length(context, &value.height)?,
            fit: fit(context, &value.fit)?,
        });
    }
    Some(())
}

pub(super) fn transform(
    context: &mut Context,
    target: &mut VisualProperties,
    value: &TransformModifierDecl,
) -> Option<()> {
    if let Some(value) = &value.position {
        target.transform.position = animation::parameter(context, value, point)?;
    }
    if let Some(value) = &value.scale {
        target.transform.scale = animation::parameter(context, value, |context, value| {
            vector(context, value, scale)
        })?;
    }
    if let Some(value) = &value.shear {
        target.transform.shear = vector(context, value, unitless)?;
    }
    if let Some(value) = &value.rotation {
        target.transform.rotation_degrees = animation::scalar(context, value, "deg")?;
    }
    if let Some(value) = &value.anchor {
        target.transform.anchor = vector(context, value, unitless)?;
    }
    if let Some(value) = &value.crop {
        target.transform.crop = Some(animation::parameter(context, value, rect)?);
    }
    if let Some(value) = &value.flip_horizontal {
        target.transform.flip_horizontal = value.value;
    }
    if let Some(value) = &value.flip_vertical {
        target.transform.flip_vertical = value.value;
    }
    Some(())
}

pub(super) fn composite(
    context: &mut Context,
    target: &mut VisualProperties,
    value: &CompositeModifierDecl,
) -> Option<()> {
    if let Some(value) = &value.opacity {
        target.opacity = animation::parameter(context, value, value::scale)?;
    }
    if let Some(value) = &value.z_index {
        target.compositing.z_index = value::integer_i32(context, value, "z-index")?;
    }
    if let Some(value) = &value.blend {
        target.compositing.blend_mode = blend(context, value)?;
    }
    Some(())
}

fn point(context: &mut Context, value: &crate::authoring::PointDecl) -> Option<Point> {
    Some(Point {
        x: length(context, &value.x)?,
        y: length(context, &value.y)?,
    })
}

fn vector(
    context: &mut Context,
    value: &crate::authoring::VectorDecl,
    convert: fn(&mut Context, &crate::authoring::NumberLiteral) -> Option<f64>,
) -> Option<Vec2> {
    Some(Vec2 {
        x: convert(context, &value.x)?,
        y: convert(context, &value.y)?,
    })
}

fn pixels(context: &mut Context, value: &crate::authoring::NumberLiteral) -> Option<f64> {
    value::scalar(context, value, "px")
}

fn scale(context: &mut Context, number: &crate::authoring::NumberLiteral) -> Option<f64> {
    value::scale(context, number)
}

fn unitless(context: &mut Context, number: &crate::authoring::NumberLiteral) -> Option<f64> {
    value::unitless(context, number)
}

fn rect(context: &mut Context, value: &crate::authoring::RectDecl) -> Option<Rect> {
    Some(Rect {
        x: normalized(context, &value.x)?,
        y: normalized(context, &value.y)?,
        width: normalized(context, &value.width)?,
        height: normalized(context, &value.height)?,
    })
}

fn normalized(context: &mut Context, value: &crate::authoring::NumberLiteral) -> Option<f64> {
    value::scale(context, value)
}

fn length(context: &mut Context, value: &crate::authoring::NumberLiteral) -> Option<Length> {
    let (_, suffix) = value::split(&value.raw);
    let number = value::number(context, value, &["px", "%"], "length")?;
    Some(Length {
        value: number,
        unit: if suffix == "%" {
            LengthUnit::Percent
        } else {
            LengthUnit::Pixels
        },
    })
}

fn anchor_value(context: &mut Context, value: &crate::authoring::Identifier) -> Option<Anchor> {
    match value.value.as_str() {
        "center" => Some(Anchor::Center),
        "top-left" => Some(Anchor::TopLeft),
        "top" => Some(Anchor::Top),
        "top-right" => Some(Anchor::TopRight),
        "left" => Some(Anchor::Left),
        "right" => Some(Anchor::Right),
        "bottom-left" => Some(Anchor::BottomLeft),
        "bottom" => Some(Anchor::Bottom),
        "bottom-right" => Some(Anchor::BottomRight),
        _ => invalid(context, "unknown anchor", value),
    }
}

fn fit(context: &mut Context, value: &crate::authoring::Identifier) -> Option<FitMode> {
    match value.value.as_str() {
        "fill" => Some(FitMode::Fill),
        "contain" => Some(FitMode::Contain),
        "cover" => Some(FitMode::Cover),
        _ => invalid(context, "fit must be fill, contain, or cover", value),
    }
}

pub(super) fn blend(
    context: &mut Context,
    value: &crate::authoring::Identifier,
) -> Option<BlendMode> {
    let result = match value.value.as_str() {
        "normal" => BlendMode::Normal,
        "multiply" => BlendMode::Multiply,
        "screen" => BlendMode::Screen,
        "overlay" => BlendMode::Overlay,
        "darken" => BlendMode::Darken,
        "lighten" => BlendMode::Lighten,
        "color-dodge" => BlendMode::ColorDodge,
        "color-burn" => BlendMode::ColorBurn,
        "hard-light" => BlendMode::HardLight,
        "soft-light" => BlendMode::SoftLight,
        "difference" => BlendMode::Difference,
        "exclusion" => BlendMode::Exclusion,
        _ => return invalid(context, "unknown blend mode", value),
    };
    Some(result)
}

fn invalid<T>(
    context: &mut Context,
    message: &str,
    value: &crate::authoring::Identifier,
) -> Option<T> {
    context.error("AUTHORING_LOWER_ENUM", message, value.span);
    None
}
