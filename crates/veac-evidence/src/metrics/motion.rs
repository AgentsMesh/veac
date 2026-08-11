use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{DiffChannels, FrameObservation, MotionMetric, PixelFormat, RegionSpec};

use super::{diff_stats, resolve_region, MetricError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MotionStats {
    pub interval_motion: Vec<f64>,
    pub first_to_last_ratio: f64,
    pub monotonically_decelerating: bool,
    pub passes_minimums: bool,
}

pub fn motion_deceleration(
    frames: &[&FrameObservation],
    region: Option<&RegionSpec>,
    metric: MotionMetric,
    minimum_interval_motion: f64,
    minimum_deceleration_ratio: f64,
    monotonic_tolerance: f64,
) -> Result<MotionStats, MetricError> {
    if frames.len() < 3
        || !minimum_interval_motion.is_finite()
        || !minimum_deceleration_ratio.is_finite()
        || !monotonic_tolerance.is_finite()
        || minimum_interval_motion < 0.0
        || minimum_deceleration_ratio < 1.0
        || monotonic_tolerance < 0.0
    {
        return Err(MetricError::InvalidThreshold);
    }
    let interval_motion = match metric {
        MotionMetric::ChangedFraction { threshold } => frames
            .windows(2)
            .map(|pair| {
                diff_stats(pair[0], pair[1], region, DiffChannels::Rgb, threshold)
                    .map(|value| value.changed_fraction)
            })
            .collect::<Result<Vec<_>, _>>()?,
        MotionMetric::AlphaCentroid { minimum } => {
            let positions = frames
                .iter()
                .map(|frame| centroid(frame, region, minimum))
                .collect::<Result<Vec<_>, _>>()?;
            positions
                .windows(2)
                .map(|pair| (pair[1].0 - pair[0].0).hypot(pair[1].1 - pair[0].1))
                .collect()
        }
    };
    let first = interval_motion[0];
    let last = *interval_motion.last().unwrap();
    let ratio = if last == 0.0 { f64::MAX } else { first / last };
    let monotonic = interval_motion
        .windows(2)
        .all(|pair| pair[0] + monotonic_tolerance >= pair[1]);
    let minimums = interval_motion
        .iter()
        .all(|value| *value >= minimum_interval_motion)
        && ratio >= minimum_deceleration_ratio;
    Ok(MotionStats {
        interval_motion,
        first_to_last_ratio: ratio,
        monotonically_decelerating: monotonic,
        passes_minimums: minimums,
    })
}

fn centroid(
    frame: &FrameObservation,
    region: Option<&RegionSpec>,
    minimum: u8,
) -> Result<(f64, f64), MetricError> {
    if frame.format != PixelFormat::Rgba8 {
        return Err(MetricError::MissingAlpha);
    }
    let rect = resolve_region(frame, region)?;
    let mut total = 0_f64;
    let mut x_total = 0_f64;
    let mut y_total = 0_f64;
    for index in rect.indices(frame.width) {
        let alpha = frame.rgba(index)[3];
        if alpha < minimum {
            continue;
        }
        let x = index as u32 % frame.width;
        let y = index as u32 / frame.width;
        let weight = f64::from(alpha);
        total += weight;
        x_total += f64::from(x) * weight;
        y_total += f64::from(y) * weight;
    }
    if total == 0.0 {
        return Err(MetricError::EmptySelection);
    }
    Ok((x_total / total, y_total / total))
}
