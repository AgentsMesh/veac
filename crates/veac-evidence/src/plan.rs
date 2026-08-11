use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    canonical_json, AssertionSpec, BundleError, PixelFormat, RationalTime, ValidatedSuite,
};

pub const OBSERVATION_PLAN_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObservationPlanV1 {
    pub schema_version: u32,
    pub suite_sha256: String,
    pub frames: Vec<PlannedFrame>,
    pub decodes: Vec<PlannedDecode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlannedFrame {
    pub source_id: String,
    pub at: RationalTime,
    pub format: PixelFormat,
    pub sample_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlannedDecode {
    pub source_id: String,
}

pub fn plan_observations(suite: &ValidatedSuite) -> Result<ObservationPlanV1, BundleError> {
    let suite = suite.as_suite();
    let required = required_samples(&suite.assertions);
    let mut frames: BTreeMap<(String, i64, u32), Vec<String>> = BTreeMap::new();
    for sample in suite
        .samples
        .iter()
        .filter(|sample| required.contains(sample.id.as_str()))
    {
        let (value, scale) = normalized_time(sample.at);
        frames
            .entry((sample.source_id.clone(), value, scale))
            .or_default()
            .push(sample.id.clone());
    }
    let frames = frames
        .into_iter()
        .map(|((source_id, value, timescale), mut sample_ids)| {
            sample_ids.sort();
            PlannedFrame {
                source_id,
                at: RationalTime { value, timescale },
                format: PixelFormat::Rgba8,
                sample_ids,
            }
        })
        .collect();
    let decodes = suite
        .assertions
        .iter()
        .filter_map(|value| match value {
            AssertionSpec::DecodeComplete(spec) => Some(spec.source_id.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|source_id| PlannedDecode { source_id })
        .collect();
    Ok(ObservationPlanV1 {
        schema_version: OBSERVATION_PLAN_SCHEMA_VERSION,
        suite_sha256: crate::sha256_hex(&canonical_json(suite)?),
        frames,
        decodes,
    })
}

fn required_samples(assertions: &[AssertionSpec]) -> BTreeSet<&str> {
    let mut ids = BTreeSet::new();
    for value in assertions {
        match value {
            AssertionSpec::DecodeComplete(_) | AssertionSpec::LayerOrder(_) => {}
            AssertionSpec::Alpha(value) => insert(&mut ids, [&value.sample_id]),
            AssertionSpec::PixelDiff(value) => {
                insert(&mut ids, [&value.left_sample_id, &value.right_sample_id])
            }
            AssertionSpec::Bounds(value) => {
                insert(&mut ids, [&value.sample_id]);
                if let crate::MaskSpec::Difference {
                    reference_sample_id,
                    ..
                } = &value.mask
                {
                    insert(&mut ids, [reference_sample_id]);
                }
            }
            AssertionSpec::CompositeOver(value) => insert(
                &mut ids,
                [
                    &value.actual_sample_id,
                    &value.underlay_sample_id,
                    &value.overlay_sample_id,
                ],
            ),
            AssertionSpec::RevealOrder(value) => {
                insert(&mut ids, [&value.baseline_sample_id]);
                insert(
                    &mut ids,
                    value.checkpoints.iter().map(|item| &item.sample_id),
                );
            }
            AssertionSpec::MotionProfile(value) => insert(&mut ids, value.sample_ids.iter()),
        }
    }
    ids
}

fn insert<'a>(set: &mut BTreeSet<&'a str>, values: impl IntoIterator<Item = &'a String>) {
    set.extend(values.into_iter().map(String::as_str));
}

fn normalized_time(value: RationalTime) -> (i64, u32) {
    let divisor = gcd(value.value.unsigned_abs(), u64::from(value.timescale));
    (
        value.value / divisor as i64,
        value.timescale / divisor as u32,
    )
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left.max(1)
}
