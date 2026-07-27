use crate::authoring::{
    CardinalDirection, CircleDirection, FadeColor, MatteMode, MatteRelation, TransitionAlignment,
    TransitionRelation, TransitionStyle, ZoomDirection,
};
use veac_ir::{RelationKind, Transition};

use super::context::Context;
use super::{relation, value};

pub(super) fn transition(
    ctx: &mut Context,
    declaration: &TransitionRelation,
) -> Option<RelationKind> {
    let transition = Transition {
        kind: style(ctx, &declaration.style)?,
        duration: value::time(ctx, &declaration.timing.duration)?,
        alignment: alignment(declaration.timing.alignment.value),
    };
    Some(RelationKind::Transition {
        from: relation::item_endpoint(ctx, &declaration.endpoints.from)?,
        to: relation::item_endpoint(ctx, &declaration.endpoints.to)?,
        transition,
    })
}

pub(super) fn matte(
    ctx: &mut Context,
    declaration: &MatteRelation,
    sequence: &veac_ir::Sequence,
) -> Option<RelationKind> {
    let producer = relation::item_id(ctx, &declaration.endpoints.producer)?;
    let consumer = relation::item_id(ctx, &declaration.endpoints.consumer)?;
    let mode = match declaration.style.mode.value {
        MatteMode::Alpha => veac_ir::TrackMatteMode::Alpha,
        MatteMode::Luma => veac_ir::TrackMatteMode::Luma,
    };
    let Some(clip) = relation::item(sequence, &consumer) else {
        ctx.error(
            "AUTHORING_LOWER_RELATION_ITEM",
            "matte consumer item does not exist in its sequence",
            declaration.endpoints.consumer.id.span,
        );
        return None;
    };
    if clip.visual.is_none() {
        ctx.error(
            "AUTHORING_LOWER_RELATION_COMPONENT",
            "matte consumer must have a visual component",
            declaration.endpoints.consumer.id.span,
        );
        return None;
    };
    let _ = producer;
    Some(RelationKind::Matte {
        producer: relation::item_endpoint(ctx, &declaration.endpoints.producer)?,
        consumer: relation::item_endpoint(ctx, &declaration.endpoints.consumer)?,
        parameters: veac_ir::MatteRelationParameters {
            mode,
            invert: declaration.style.invert.value,
        },
    })
}

fn style(ctx: &mut Context, value: &TransitionStyle) -> Option<veac_ir::TransitionKind> {
    Some(match value {
        TransitionStyle::Dissolve => veac_ir::TransitionKind::Dissolve,
        TransitionStyle::Fade { color } => veac_ir::TransitionKind::Fade {
            color: fade_color(color.value),
        },
        TransitionStyle::Wipe {
            direction,
            angle,
            softness,
        } => veac_ir::TransitionKind::Wipe {
            direction: cardinal(direction.value),
            angle_degrees: value::scalar(ctx, angle, "deg")?,
            softness: value::scale(ctx, softness)?,
        },
        TransitionStyle::Slide { direction, amount } => veac_ir::TransitionKind::Slide {
            direction: cardinal(direction.value),
            amount: value::unitless(ctx, amount)?,
        },
        TransitionStyle::Zoom { direction, amount } => veac_ir::TransitionKind::Zoom {
            direction: match direction.value {
                ZoomDirection::In => veac_ir::ZoomDirection::In,
                ZoomDirection::Out => veac_ir::ZoomDirection::Out,
            },
            amount: value::unitless(ctx, amount)?,
        },
        TransitionStyle::Circle {
            direction,
            softness,
        } => veac_ir::TransitionKind::Circle {
            direction: match direction.value {
                CircleDirection::Open => veac_ir::CircleDirection::Open,
                CircleDirection::Close => veac_ir::CircleDirection::Close,
            },
            softness: value::scale(ctx, softness)?,
        },
        TransitionStyle::Pixelize { amount } => veac_ir::TransitionKind::Pixelize {
            amount: value::scale(ctx, amount)?,
        },
    })
}

fn cardinal(value: CardinalDirection) -> veac_ir::CardinalDirection {
    match value {
        CardinalDirection::Left => veac_ir::CardinalDirection::Left,
        CardinalDirection::Right => veac_ir::CardinalDirection::Right,
        CardinalDirection::Up => veac_ir::CardinalDirection::Up,
        CardinalDirection::Down => veac_ir::CardinalDirection::Down,
    }
}

fn fade_color(value: FadeColor) -> veac_ir::FadeColor {
    match value {
        FadeColor::Transparent => veac_ir::FadeColor::Transparent,
        FadeColor::Black => veac_ir::FadeColor::Black,
        FadeColor::White => veac_ir::FadeColor::White,
    }
}

fn alignment(value: TransitionAlignment) -> veac_ir::TransitionAlignment {
    match value {
        TransitionAlignment::BeforeCut => veac_ir::TransitionAlignment::BeforeCut,
        TransitionAlignment::Centered => veac_ir::TransitionAlignment::Centered,
        TransitionAlignment::AfterCut => veac_ir::TransitionAlignment::AfterCut,
    }
}
