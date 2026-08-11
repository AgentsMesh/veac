use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{DiffChannels, FrameObservation, RegionSpec, RevealCheckpoint};

use super::{diff_stats, MetricError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RevealStats {
    pub change_fractions: Vec<Vec<f64>>,
    pub matches_expected_prefix: bool,
}

pub fn reveal_prefix(
    baseline: &FrameObservation,
    checkpoints: &[(&RevealCheckpoint, &FrameObservation)],
    regions: &[&RegionSpec],
    change_threshold: u8,
    visible_minimum: f64,
    hidden_maximum: f64,
) -> Result<RevealStats, MetricError> {
    if checkpoints.is_empty() || regions.is_empty() || hidden_maximum >= visible_minimum {
        return Err(MetricError::InvalidThreshold);
    }
    let mut matches = true;
    let mut matrix = Vec::with_capacity(checkpoints.len());
    for (checkpoint, frame) in checkpoints {
        if checkpoint.visible_prefix > regions.len() {
            return Err(MetricError::InvalidThreshold);
        }
        let mut row = Vec::with_capacity(regions.len());
        for (index, region) in regions.iter().enumerate() {
            let fraction = diff_stats(
                baseline,
                frame,
                Some(region),
                DiffChannels::Rgb,
                change_threshold,
            )?
            .changed_fraction;
            let expected = if index < checkpoint.visible_prefix {
                fraction >= visible_minimum
            } else {
                fraction <= hidden_maximum
            };
            matches &= expected;
            row.push(fraction);
        }
        matrix.push(row);
    }
    Ok(RevealStats {
        change_fractions: matrix,
        matches_expected_prefix: matches,
    })
}
