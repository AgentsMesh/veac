use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{FrameObservation, PixelFormat, RegionSpec};

use super::{resolve_region, MetricError};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AlphaStats {
    pub minimum: u8,
    pub maximum: u8,
    pub mean: f64,
    pub transparent_fraction: f64,
    pub partial_fraction: f64,
    pub opaque_fraction: f64,
}

pub fn alpha_stats(
    frame: &FrameObservation,
    region: Option<&RegionSpec>,
    transparent_below: u8,
    opaque_above: u8,
) -> Result<AlphaStats, MetricError> {
    if frame.format != PixelFormat::Rgba8 {
        return Err(MetricError::MissingAlpha);
    }
    if transparent_below >= opaque_above {
        return Err(MetricError::InvalidThreshold);
    }
    let rect = resolve_region(frame, region)?;
    let mut minimum = u8::MAX;
    let mut maximum = 0_u8;
    let mut total = 0_u64;
    let mut transparent = 0_u64;
    let mut opaque = 0_u64;
    let mut count = 0_u64;
    for index in rect.indices(frame.width) {
        let alpha = frame.rgba(index)[3];
        minimum = minimum.min(alpha);
        maximum = maximum.max(alpha);
        total += u64::from(alpha);
        transparent += u64::from(alpha <= transparent_below);
        opaque += u64::from(alpha >= opaque_above);
        count += 1;
    }
    let denominator = count as f64;
    Ok(AlphaStats {
        minimum,
        maximum,
        mean: total as f64 / denominator / 255.0,
        transparent_fraction: transparent as f64 / denominator,
        partial_fraction: (count - transparent - opaque) as f64 / denominator,
        opaque_fraction: opaque as f64 / denominator,
    })
}
