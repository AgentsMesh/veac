use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{DiffChannels, FrameObservation, RegionSpec};

use super::frame::same_geometry;
use super::{resolve_region, MetricError};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiffStats {
    pub rmse: f64,
    pub mae: f64,
    pub maximum_delta: f64,
    pub changed_fraction: f64,
}

pub fn diff_stats(
    left: &FrameObservation,
    right: &FrameObservation,
    region: Option<&RegionSpec>,
    channels: DiffChannels,
    change_threshold: u8,
) -> Result<DiffStats, MetricError> {
    same_geometry(left, right)?;
    let rect = resolve_region(left, region)?;
    let selected = selected_channels(channels);
    let mut squared = 0_f64;
    let mut absolute = 0_u64;
    let mut maximum = 0_u8;
    let mut changed = 0_u64;
    let mut values = 0_u64;
    let mut pixels = 0_u64;
    for index in rect.indices(left.width) {
        let a = left.rgba(index);
        let b = right.rgba(index);
        let mut pixel_changed = false;
        for channel in selected {
            let delta = a[*channel].abs_diff(b[*channel]);
            squared += f64::from(delta).powi(2);
            absolute += u64::from(delta);
            maximum = maximum.max(delta);
            pixel_changed |= delta > change_threshold;
            values += 1;
        }
        changed += u64::from(pixel_changed);
        pixels += 1;
    }
    let denominator = values as f64;
    Ok(DiffStats {
        rmse: (squared / denominator).sqrt() / 255.0,
        mae: absolute as f64 / denominator / 255.0,
        maximum_delta: f64::from(maximum) / 255.0,
        changed_fraction: changed as f64 / pixels as f64,
    })
}

fn selected_channels(channels: DiffChannels) -> &'static [usize] {
    match channels {
        DiffChannels::Rgb => &[0, 1, 2],
        DiffChannels::Rgba => &[0, 1, 2, 3],
        DiffChannels::Alpha => &[3],
    }
}
