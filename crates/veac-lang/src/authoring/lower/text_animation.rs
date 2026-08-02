use crate::authoring::{
    PointDecl, TextAnimationDecl, TextAnimationTransformDecl, TextGranularityDecl, VectorDecl,
};
use veac_ir::{
    Animatable, Point, TextAnimation, TextGranularity, TextHighlightAnimation, TextUnitTransform,
    Vec2,
};

use super::context::Context;
use super::{animation, value};

pub(super) fn lower(ctx: &mut Context, value: &TextAnimationDecl) -> Option<TextAnimation> {
    Some(TextAnimation {
        granularity: granularity(value.unit.value),
        stagger: super::value::time(ctx, &value.stagger)?,
        reveal: match &value.reveal {
            Some(value) => animation::parameter(ctx, value, value::scale)?,
            None => Animatable::constant(1.0),
        },
        highlight: match &value.highlight {
            Some(highlight) => Some(TextHighlightAnimation {
                fill: super::color::lower(ctx, &highlight.fill)?,
                progress: animation::parameter(ctx, &highlight.progress, value::scale)?,
            }),
            None => None,
        },
        opacity: match &value.opacity {
            Some(value) => animation::parameter(ctx, value, value::scale)?,
            None => Animatable::constant(1.0),
        },
        transform: match &value.transform {
            Some(value) => transform(ctx, value)?,
            None => TextUnitTransform::default(),
        },
    })
}

fn granularity(value: TextGranularityDecl) -> TextGranularity {
    match value {
        TextGranularityDecl::Whole => TextGranularity::Whole,
        TextGranularityDecl::Line => TextGranularity::Line,
        TextGranularityDecl::Word => TextGranularity::Word,
        TextGranularityDecl::Grapheme => TextGranularity::Grapheme,
    }
}

fn transform(ctx: &mut Context, value: &TextAnimationTransformDecl) -> Option<TextUnitTransform> {
    let default = TextUnitTransform::default();
    Some(TextUnitTransform {
        position_offset: match &value.position {
            Some(value) => animation::parameter(ctx, value, point)?,
            None => default.position_offset,
        },
        scale: match &value.scale {
            Some(value) => animation::parameter(ctx, value, scale)?,
            None => default.scale,
        },
        rotation_degrees: match &value.rotation {
            Some(value) => animation::scalar(ctx, value, "deg")?,
            None => default.rotation_degrees,
        },
    })
}

fn point(ctx: &mut Context, value: &PointDecl) -> Option<Point> {
    Some(Point {
        x: super::text_layout::length(ctx, &value.x)?,
        y: super::text_layout::length(ctx, &value.y)?,
    })
}

fn scale(ctx: &mut Context, vector: &VectorDecl) -> Option<Vec2> {
    Some(Vec2 {
        x: value::scale(ctx, &vector.x)?,
        y: value::scale(ctx, &vector.y)?,
    })
}
