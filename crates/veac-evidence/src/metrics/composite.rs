use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{FrameObservation, PixelFormat, RegionSpec};

use super::frame::same_geometry;
use super::{resolve_region, MetricError};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CompositeStats {
    pub expected_rmse: f64,
    pub underlay_rmse: f64,
    pub improvement_ratio: f64,
    pub selected_fraction: f64,
}

pub fn source_over(underlay: [u8; 4], overlay: [u8; 4]) -> [u8; 4] {
    let source_alpha = f64::from(overlay[3]) / 255.0;
    let base_alpha = f64::from(underlay[3]) / 255.0;
    let output_alpha = source_alpha + base_alpha * (1.0 - source_alpha);
    let mut output = [0_u8; 4];
    for channel in 0..3 {
        let premultiplied = f64::from(overlay[channel]) * source_alpha
            + f64::from(underlay[channel]) * base_alpha * (1.0 - source_alpha);
        output[channel] = if output_alpha == 0.0 {
            0
        } else {
            (premultiplied / output_alpha).round().clamp(0.0, 255.0) as u8
        };
    }
    output[3] = (output_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
    output
}

pub fn compare_composite(
    actual: &FrameObservation,
    underlay: &FrameObservation,
    overlay: &FrameObservation,
    region: Option<&RegionSpec>,
    alpha_minimum: u8,
) -> Result<CompositeStats, MetricError> {
    same_geometry(actual, underlay)?;
    same_geometry(actual, overlay)?;
    if overlay.format != PixelFormat::Rgba8 {
        return Err(MetricError::MissingAlpha);
    }
    let rect = resolve_region(actual, region)?;
    let mut expected_squared = 0_f64;
    let mut underlay_squared = 0_f64;
    let mut selected = 0_u64;
    for index in rect.indices(actual.width) {
        let top = overlay.rgba(index);
        if top[3] < alpha_minimum {
            continue;
        }
        let observed = actual.rgba(index);
        let base = underlay.rgba(index);
        let expected = source_over(base, top);
        for channel in 0..3 {
            expected_squared += f64::from(observed[channel].abs_diff(expected[channel])).powi(2);
            underlay_squared += f64::from(observed[channel].abs_diff(base[channel])).powi(2);
        }
        selected += 1;
    }
    if selected == 0 {
        return Err(MetricError::EmptySelection);
    }
    let denominator = (selected * 3) as f64;
    let expected_rmse = (expected_squared / denominator).sqrt() / 255.0;
    let underlay_rmse = (underlay_squared / denominator).sqrt() / 255.0;
    Ok(CompositeStats {
        expected_rmse,
        underlay_rmse,
        improvement_ratio: if expected_rmse == 0.0 {
            f64::MAX
        } else {
            underlay_rmse / expected_rmse
        },
        selected_fraction: selected as f64 / f64::from(rect.width * rect.height),
    })
}
