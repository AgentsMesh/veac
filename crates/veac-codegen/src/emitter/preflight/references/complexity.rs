use std::collections::BTreeMap;

use veac_plan::canonical::SequenceId;
use veac_plan::{ResolvedClipSource, ResolvedRenderPlan, ResolvedSourceTimeMap};

use super::super::Check;

const MAX_EXPANDED_FILTER_UNITS: usize = 16_384;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    let mut costs: BTreeMap<&SequenceId, usize> = BTreeMap::new();
    let mut depths: BTreeMap<&SequenceId, usize> = BTreeMap::new();
    for sequence in &plan.sequences {
        let mut cost = 1usize;
        let mut depth = 1usize;
        for clip in sequence.tracks.iter().flat_map(|track| &track.clips) {
            let base = match &clip.source {
                ResolvedClipSource::Sequence { sequence_id } => {
                    depth = depth.max(depths.get(sequence_id).copied().unwrap_or(0) + 1);
                    costs.get(sequence_id).copied().unwrap_or(1)
                }
                _ => 1,
            };
            let repeat = clip
                .source_mapping
                .as_ref()
                .and_then(|mapping| match mapping.time_map {
                    ResolvedSourceTimeMap::Linear { repeat, .. } => Some(repeat as usize),
                    ResolvedSourceTimeMap::Curve { .. } => None,
                })
                .unwrap_or(1);
            let unit = base
                .saturating_mul(repeat)
                .min(MAX_EXPANDED_FILTER_UNITS + 1);
            cost = bounded_add(cost, unit);
        }
        costs.insert(&sequence.id, cost);
        depths.insert(&sequence.id, depth);
    }
    if costs
        .get(&plan.entry_sequence_id)
        .is_some_and(|cost| *cost > MAX_EXPANDED_FILTER_UNITS)
    {
        check.push(
            "PLAN_EXPANSION_LIMIT",
            Some(plan.entry_sequence_id.to_string()),
            "nested sequences and source repeats exceed the backend expansion budget",
        );
    }
    if depths
        .get(&plan.entry_sequence_id)
        .is_some_and(|depth| *depth > veac_plan::canonical::MAX_SEQUENCE_NESTING_DEPTH)
    {
        check.push(
            "PLAN_SEQUENCE_DEPTH_LIMIT",
            Some(plan.entry_sequence_id.to_string()),
            "nested sequence depth exceeds the backend recursion budget",
        );
    }
}

fn bounded_add(left: usize, right: usize) -> usize {
    left.saturating_add(right)
        .min(MAX_EXPANDED_FILTER_UNITS + 1)
}
