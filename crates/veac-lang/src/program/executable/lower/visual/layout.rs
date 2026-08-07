use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{Anchor, FitMode, Frame, Placement, Transform2D};

use super::super::animation;
use super::{channel, malformed, ExecutableLowerError};
use crate::program::executable::lower::value;

pub(super) struct Layout {
    pub(super) placement: Placement,
    pub(super) frame: Option<Frame>,
    pub(super) transform: Transform2D,
}

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    scope: &[&str],
) -> Result<Layout, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::VisualLayout)?;
    if values.len() != 3 {
        return Err(malformed());
    }
    Ok(Layout {
        placement: placement(graph, &values[0])?,
        frame: frame(graph, &values[1])?,
        transform: transform(graph, &values[2], timebase, scope)?,
    })
}

fn placement(graph: &FrozenDomainGraph, source: &Value) -> Result<Placement, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::PlacementAnchor, [anchor, inset]) => Ok(Placement::Anchor {
            anchor: anchor_value(graph, anchor)?,
            inset: animation::vector_value(graph, Some(inset))?,
        }),
        (Op::PlacementAbsolute, [position]) => Ok(Placement::Absolute {
            position: animation::point_value(graph, Some(position))?,
        }),
        _ => Err(malformed()),
    }
}

fn frame(graph: &FrozenDomainGraph, source: &Value) -> Result<Option<Frame>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::FrameNone, []) => Ok(None),
        (Op::FrameSized, [width, height, fit]) => Ok(Some(Frame {
            width: animation::length_value(graph, Some(width))?,
            height: animation::length_value(graph, Some(height))?,
            fit: fit_value(graph, fit)?,
        })),
        _ => Err(malformed()),
    }
}

fn transform(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    scope: &[&str],
) -> Result<Transform2D, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::Transform2d)?;
    if values.len() != 2 {
        return Err(malformed());
    }
    let motion = value::description_operands(graph, &values[0], Op::TransformMotion)?;
    let geometry = value::description_operands(graph, &values[1], Op::TransformGeometry)?;
    if motion.len() != 3 || geometry.len() != 4 {
        return Err(malformed());
    }
    let (flip_horizontal, flip_vertical) = flip(graph, &geometry[1])?;
    Ok(Transform2D {
        position: animation::point(
            graph,
            &motion[0],
            timebase,
            &channel(scope, "visual.position"),
        )?,
        scale: animation::vector(graph, &motion[1], timebase, &channel(scope, "visual.scale"))?,
        rotation_degrees: animation::angle(
            graph,
            &motion[2],
            timebase,
            &channel(scope, "visual.rotation"),
        )?,
        shear: animation::vector_value(graph, geometry.first())?,
        flip_horizontal,
        flip_vertical,
        anchor: animation::vector_value(graph, geometry.get(2))?,
        crop: crop(graph, &geometry[3], timebase, scope)?,
    })
}

fn crop(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    scope: &[&str],
) -> Result<Option<veac_ir::Animatable<veac_ir::Rect>>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::CropNone, []) => Ok(None),
        (Op::CropAnimated, [value]) => Ok(Some(animation::rect(
            graph,
            value,
            timebase,
            &channel(scope, "visual.crop"),
        )?)),
        _ => Err(malformed()),
    }
}

fn anchor_value(graph: &FrozenDomainGraph, source: &Value) -> Result<Anchor, ExecutableLowerError> {
    Ok(match empty(graph, source)? {
        Op::AnchorCenter => Anchor::Center,
        Op::AnchorTopLeft => Anchor::TopLeft,
        Op::AnchorTop => Anchor::Top,
        Op::AnchorTopRight => Anchor::TopRight,
        Op::AnchorLeft => Anchor::Left,
        Op::AnchorRight => Anchor::Right,
        Op::AnchorBottomLeft => Anchor::BottomLeft,
        Op::AnchorBottom => Anchor::Bottom,
        Op::AnchorBottomRight => Anchor::BottomRight,
        _ => return Err(malformed()),
    })
}

fn fit_value(graph: &FrozenDomainGraph, source: &Value) -> Result<FitMode, ExecutableLowerError> {
    Ok(match empty(graph, source)? {
        Op::FitFill => FitMode::Fill,
        Op::FitContain => FitMode::Contain,
        Op::FitCover => FitMode::Cover,
        _ => return Err(malformed()),
    })
}

fn flip(graph: &FrozenDomainGraph, source: &Value) -> Result<(bool, bool), ExecutableLowerError> {
    Ok(match empty(graph, source)? {
        Op::FlipNone => (false, false),
        Op::FlipHorizontal => (true, false),
        Op::FlipVertical => (false, true),
        Op::FlipBoth => (true, true),
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
