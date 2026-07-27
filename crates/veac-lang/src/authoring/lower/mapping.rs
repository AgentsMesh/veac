use crate::authoring::{
    InterpolationKind, MappingDecl, MappingKey, NumberLiteral, SourceOutOfRangeDecl,
};
use veac_ir::{
    FrameSynthesisPolicy, PlaybackDirection, Rational, SourceMapping, SourceOutOfRangePolicy,
    SourceTimeInterpolation, SourceTimeMap, SourceTimeSegment,
};

use super::{context::Context, value};

pub fn lower(
    ctx: &mut Context,
    mapping: Option<&MappingDecl>,
    record_duration: &NumberLiteral,
) -> Option<SourceMapping> {
    let duration = value::time(ctx, record_duration)?;
    let out_of_range = mapping.map_or(SourceOutOfRangePolicy::Strict, |mapping| {
        lower_outside(match mapping {
            MappingDecl::Linear { outside, .. } | MappingDecl::Curve { outside, .. } => *outside,
            MappingDecl::Freeze { .. } => SourceOutOfRangeDecl::Strict,
        })
    });
    let time_map = match mapping {
        None => SourceTimeMap::Linear {
            source_start: veac_ir::RationalTime::zero(ctx.timescale).ok()?,
            rate: Rational::new(1, 1).expect("one is a valid rational"),
            repeat: 1,
            direction: PlaybackDirection::Forward,
        },
        Some(MappingDecl::Linear { from, to, span, .. }) => {
            let from = value::time(ctx, from)?;
            let to = value::time(ctx, to)?;
            let delta = to.value.checked_sub(from.value)?;
            if delta == 0 {
                ctx.error(
                    "AUTHORING_LOWER_MAPPING",
                    "linear mapping source range cannot be empty",
                    *span,
                );
                return None;
            }
            SourceTimeMap::Linear {
                source_start: if delta < 0 { to } else { from },
                rate: ratio(ctx, delta.unsigned_abs(), duration.value, *span)?,
                repeat: 1,
                direction: if delta < 0 {
                    PlaybackDirection::Reverse
                } else {
                    PlaybackDirection::Forward
                },
            }
        }
        Some(MappingDecl::Curve { keys, span, .. }) => SourceTimeMap::Curve {
            segments: curve(ctx, keys, duration, *span)?,
        },
        Some(MappingDecl::Freeze { source, .. }) => SourceTimeMap::Curve {
            segments: vec![SourceTimeSegment {
                record_duration: duration,
                source_start: value::time(ctx, source)?,
                source_end: value::time(ctx, source)?,
                interpolation: SourceTimeInterpolation::Hold,
            }],
        },
    };
    Some(SourceMapping {
        time_map,
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range,
    })
}

fn lower_outside(value: SourceOutOfRangeDecl) -> SourceOutOfRangePolicy {
    match value {
        SourceOutOfRangeDecl::Strict => SourceOutOfRangePolicy::Strict,
        SourceOutOfRangeDecl::HoldFirst => SourceOutOfRangePolicy::HoldFirst,
        SourceOutOfRangeDecl::HoldLast => SourceOutOfRangePolicy::HoldLast,
        SourceOutOfRangeDecl::HoldBoth => SourceOutOfRangePolicy::HoldBoth,
    }
}

fn curve(
    ctx: &mut Context,
    keys: &[MappingKey],
    duration: veac_ir::RationalTime,
    span: crate::authoring::Span,
) -> Option<Vec<SourceTimeSegment>> {
    if keys.len() < 2 {
        ctx.error(
            "AUTHORING_LOWER_MAPPING",
            "curve mapping requires at least two keys",
            span,
        );
        return None;
    }
    let mut points = Vec::with_capacity(keys.len());
    for key in keys {
        points.push((
            value::time(ctx, &key.at)?,
            value::time(ctx, &key.source)?,
            interpolation(ctx, &key.interpolation.kind, key.interpolation.span)?,
        ));
    }
    if points[0].0.value != 0 || points.last()?.0 != duration {
        ctx.error(
            "AUTHORING_LOWER_MAPPING_DOMAIN",
            "curve keys must cover the complete item record duration",
            span,
        );
        return None;
    }
    points
        .windows(2)
        .map(|pair| {
            let record_ticks = pair[1].0.value.checked_sub(pair[0].0.value)?;
            Some(SourceTimeSegment {
                record_duration: veac_ir::RationalTime::new(record_ticks, ctx.timescale).ok()?,
                source_start: pair[0].1,
                source_end: pair[1].1,
                interpolation: pair[0].2,
            })
        })
        .collect()
}

fn interpolation(
    ctx: &mut Context,
    value: &InterpolationKind,
    span: crate::authoring::Span,
) -> Option<SourceTimeInterpolation> {
    match value {
        InterpolationKind::Hold => Some(SourceTimeInterpolation::Hold),
        InterpolationKind::Linear => Some(SourceTimeInterpolation::Linear),
        _ => {
            ctx.unsupported("eased source-time interpolation", span);
            None
        }
    }
}

fn ratio(
    ctx: &mut Context,
    numerator: u64,
    denominator: i64,
    span: crate::authoring::Span,
) -> Option<Rational> {
    let denominator = u64::try_from(denominator).ok()?;
    let divisor = gcd(numerator, denominator);
    let numerator = i64::try_from(numerator / divisor).ok()?;
    let denominator = u32::try_from(denominator / divisor).ok();
    denominator
        .and_then(|value| Rational::new(numerator, value).ok())
        .or_else(|| {
            ctx.error(
                "AUTHORING_LOWER_RATIONAL",
                "mapping rate exceeds canonical rational range",
                span,
            );
            None
        })
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        let next = left % right;
        left = right;
        right = next;
    }
    left
}
