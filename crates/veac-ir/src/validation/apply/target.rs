use crate::*;

use super::super::Validator;

impl Validator {
    pub(super) fn apply_target(
        &mut self,
        sequence: &Sequence,
        apply: &Apply,
        path: &str,
    ) -> Option<(i32, i32)> {
        match &apply.target {
            ApplyTarget::Layer { track_id } => {
                let target = self.target_track(sequence, track_id, path, &apply.id)?;
                if !has_overlap(target, apply.record_range) {
                    self.value_error("APPLY_TARGET_RANGE", path, apply.id.as_str());
                }
                None
            }
            ApplyTarget::CompositeBand {
                from_track_id,
                through_track_id,
            } => self.composite_band(sequence, apply, from_track_id, through_track_id, path),
            ApplyTarget::ItemSet { item_ids } => {
                self.item_set(sequence, apply, item_ids, path);
                None
            }
        }
    }

    fn composite_band(
        &mut self,
        sequence: &Sequence,
        apply: &Apply,
        from_id: &TrackId,
        through_id: &TrackId,
        path: &str,
    ) -> Option<(i32, i32)> {
        let from = self.target_track(sequence, from_id, path, &apply.id)?;
        let through = self.target_track(sequence, through_id, path, &apply.id)?;
        if from.id == through.id || from.order >= through.order {
            self.value_error("APPLY_BAND_ORDER", path, apply.id.as_str());
            return None;
        }
        let overlaps = sequence.tracks.iter().any(|track| {
            composition_track(track)
                && (from.order..=through.order).contains(&track.order)
                && has_overlap(track, apply.record_range)
        });
        if !overlaps {
            self.value_error("APPLY_TARGET_RANGE", path, apply.id.as_str());
        }
        Some((from.order, through.order))
    }

    fn item_set(&mut self, sequence: &Sequence, apply: &Apply, ids: &[ItemId], path: &str) {
        if ids.is_empty() {
            self.value_error("APPLY_ITEM_SET_EMPTY", path, apply.id.as_str());
        }
        if ids.windows(2).any(|pair| pair[0] >= pair[1]) {
            self.value_error("APPLY_ITEM_SET_ORDER", path, apply.id.as_str());
        }
        for id in ids {
            self.check_id(id.is_valid(), id.as_str(), path);
            let Some((track, clip)) = find_item(sequence, id) else {
                self.missing_ref("APPLY_ITEM_NOT_FOUND", id.as_str(), path);
                continue;
            };
            if !composition_track(track) {
                self.value_error("APPLY_TARGET_TYPE", path, apply.id.as_str());
            }
            if !ranges_overlap(clip.record_range, apply.record_range) {
                self.value_error("APPLY_TARGET_RANGE", path, id.as_str());
            }
        }
    }

    fn target_track<'a>(
        &mut self,
        sequence: &'a Sequence,
        id: &TrackId,
        path: &str,
        apply_id: &ApplyId,
    ) -> Option<&'a Track> {
        self.check_id(id.is_valid(), id.as_str(), path);
        let Some(track) = sequence.tracks.iter().find(|track| track.id == *id) else {
            self.missing_ref("APPLY_TRACK_NOT_FOUND", id.as_str(), path);
            return None;
        };
        if !composition_track(track) {
            self.value_error("APPLY_TARGET_TYPE", path, apply_id.as_str());
            return None;
        }
        Some(track)
    }
}

pub(super) fn target_contains(
    sequence: &Sequence,
    target: &ApplyTarget,
    item: RelationItem<'_>,
) -> bool {
    match target {
        ApplyTarget::ItemSet { item_ids } => item_ids.contains(&item.clip.id),
        ApplyTarget::Layer { track_id } => item.track.id == *track_id,
        ApplyTarget::CompositeBand {
            from_track_id,
            through_track_id,
        } => {
            let Some(from) = sequence
                .tracks
                .iter()
                .find(|track| track.id == *from_track_id)
            else {
                return false;
            };
            let Some(through) = sequence
                .tracks
                .iter()
                .find(|track| track.id == *through_track_id)
            else {
                return false;
            };
            (from.order..=through.order).contains(&item.track.order)
        }
    }
}

fn find_item<'a>(sequence: &'a Sequence, id: &ItemId) -> Option<(&'a Track, &'a Clip)> {
    sequence.tracks.iter().find_map(|track| {
        track
            .clips
            .iter()
            .find(|clip| clip.id == *id)
            .map(|clip| (track, clip))
    })
}

fn composition_track(track: &Track) -> bool {
    matches!(
        track.kind,
        TrackKind::Video | TrackKind::Visual | TrackKind::Caption
    )
}

fn has_overlap(track: &Track, range: TimeRange) -> bool {
    track
        .clips
        .iter()
        .any(|clip| ranges_overlap(clip.record_range, range))
}

fn ranges_overlap(left: TimeRange, right: TimeRange) -> bool {
    left.end().is_ok_and(|left_end| {
        right
            .end()
            .is_ok_and(|right_end| left.start < right_end && right.start < left_end)
    })
}
