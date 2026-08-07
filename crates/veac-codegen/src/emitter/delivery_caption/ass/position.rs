use veac_plan::canonical::{
    Anchor, Animatable, BlendMode, HorizontalTextAlignment, Length, LengthUnit, Placement,
    VerticalTextAlignment,
};

use super::super::{failure::Failure, Cue};

#[derive(Clone, Copy)]
pub(super) struct Position {
    pub x: f64,
    pub y: f64,
}

pub(super) fn resolve(
    cue: &Cue<'_>,
    style: &veac_plan::ResolvedTextStyle,
    width: u32,
    height: u32,
) -> Result<Position, Failure> {
    if !cue.clip.effects.is_empty() {
        return Err(unsupported("caption clip effects"));
    }
    let visual = cue.clip.visual.as_ref().ok_or_else(|| {
        Failure::invalid(
            "CAPTION_ASS_VISUAL_MISSING",
            "caption clip has no visual properties",
        )
    })?;
    if visual.frame.is_some()
        || visual.transform.flip_horizontal
        || visual.transform.flip_vertical
        || visual.transform.shear.x != 0.0
        || visual.transform.shear.y != 0.0
        || visual.transform.crop.is_some()
        || !visual.masks.is_empty()
        || visual.track_matte.is_some()
        || visual.card.is_some()
        || visual.color_pipeline.is_some()
        || visual.compositing.blend_mode != BlendMode::Normal
    {
        return Err(unsupported("framed or composited caption visuals"));
    }
    let offset = constant(&visual.transform.position)?;
    let scale = constant(&visual.transform.scale)?;
    let rotation = constant(&visual.transform.rotation_degrees)?;
    let opacity = constant(&visual.opacity)?;
    if scale.x != 1.0 || scale.y != 1.0 || *rotation != 0.0 || *opacity != 1.0 {
        return Err(unsupported(
            "scaled, rotated, animated, or translucent caption visuals",
        ));
    }
    let extent = (f64::from(width), f64::from(height));
    let (target_x, target_y) = placement(visual.placement, extent)?;
    let left = target_x + length(offset.x, extent.0)? - visual.transform.anchor.x * extent.0;
    let top = target_y + length(offset.y, extent.1)? - visual.transform.anchor.y * extent.1;
    if !left.is_finite() || !top.is_finite() {
        return Err(invalid());
    }
    let layout = style.layout;
    Ok(Position {
        x: left
            + match layout.horizontal_alignment {
                HorizontalTextAlignment::Left => 0.0,
                HorizontalTextAlignment::Center => extent.0 / 2.0,
                HorizontalTextAlignment::Right => extent.0,
            },
        y: top
            + match layout.vertical_alignment {
                VerticalTextAlignment::Top => 0.0,
                VerticalTextAlignment::Middle => extent.1 / 2.0,
                VerticalTextAlignment::Bottom => extent.1,
            },
    })
}

fn constant<T>(value: &Animatable<T>) -> Result<&T, Failure> {
    match value {
        Animatable::Constant { value } => Ok(value),
        Animatable::Keyframes { .. } => Err(unsupported("animated caption visuals")),
        Animatable::Binding { .. } => Err(unsupported("temporal caption visuals")),
    }
}

fn placement(value: Placement, extent: (f64, f64)) -> Result<(f64, f64), Failure> {
    match value {
        Placement::Absolute { position } => {
            Ok((length(position.x, extent.0)?, length(position.y, extent.1)?))
        }
        Placement::Anchor { anchor, inset } => {
            if !inset.x.is_finite() || !inset.y.is_finite() {
                return Err(invalid());
            }
            let x = match anchor {
                Anchor::TopLeft | Anchor::Left | Anchor::BottomLeft => inset.x,
                Anchor::Top | Anchor::Center | Anchor::Bottom => extent.0 / 2.0 + inset.x,
                Anchor::TopRight | Anchor::Right | Anchor::BottomRight => extent.0 - inset.x,
            };
            let y = match anchor {
                Anchor::TopLeft | Anchor::Top | Anchor::TopRight => inset.y,
                Anchor::Left | Anchor::Center | Anchor::Right => extent.1 / 2.0 + inset.y,
                Anchor::BottomLeft | Anchor::Bottom | Anchor::BottomRight => extent.1 - inset.y,
            };
            Ok((x, y))
        }
    }
}

fn length(value: Length, extent: f64) -> Result<f64, Failure> {
    if !value.value.is_finite() {
        return Err(invalid());
    }
    Ok(match value.unit {
        LengthUnit::Pixels => value.value,
        LengthUnit::Normalized => value.value * extent,
        LengthUnit::Percent => value.value * extent / 100.0,
    })
}

fn unsupported(feature: &str) -> Failure {
    Failure::unsupported(
        "CAPTION_ASS_VISUAL_UNSUPPORTED",
        format!("ASS sidecars cannot preserve {feature}"),
    )
}

fn invalid() -> Failure {
    Failure::invalid(
        "CAPTION_ASS_VISUAL_INVALID",
        "caption visual contains invalid numeric values",
    )
}
