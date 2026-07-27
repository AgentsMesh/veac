mod overlap;
mod pipeline;
mod target;

use std::cmp::Ordering;

use crate::*;

use self::overlap::{crossing_bands, BandScope};
use super::Validator;

impl Validator {
    pub(super) fn applies(&mut self, sequence: &Sequence, timebase: u32, sequence_path: &str) {
        if sequence.applies.len() > 1_024 {
            self.value_error("APPLY_COUNT", sequence_path, sequence.id.as_str());
        }
        let duration = sequence_duration(sequence);
        let mut bands = Vec::new();
        for apply in &sequence.applies {
            let path = format!("{sequence_path}/applies/{}", apply.id);
            self.check_id(apply.id.is_valid(), apply.id.as_str(), &path);
            if !self.apply_ids.insert(apply.id.to_string()) {
                self.duplicate("DUPLICATE_APPLY_ID", apply.id.as_str(), &path);
            }
            self.time_range(
                apply.record_range,
                timebase,
                "APPLY_RANGE",
                &format!("{path}/record_range"),
                apply.id.as_str(),
            );
            if duration.is_none_or(|end| {
                apply
                    .record_range
                    .end()
                    .map_or(true, |apply_end| apply_end > end)
            }) {
                self.value_error("APPLY_RANGE", &path, apply.id.as_str());
            }
            if let Some((from_order, through_order)) = self.apply_target(sequence, apply, &path) {
                bands.push(BandScope {
                    id: &apply.id,
                    range: apply.record_range,
                    from_order,
                    through_order,
                });
            }
            self.apply_pipeline(apply, timebase, &path);
        }
        for id in crossing_bands(&bands) {
            self.value_error("APPLY_BAND_CROSSING", sequence_path, id.as_str());
        }
    }
}

fn sequence_duration(sequence: &Sequence) -> Option<RationalTime> {
    sequence
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .filter_map(|clip| clip.record_range.end().ok())
        .max_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal))
}

pub(super) fn target_contains(
    sequence: &Sequence,
    target: &ApplyTarget,
    item: RelationItem<'_>,
) -> bool {
    target::target_contains(sequence, target, item)
}
