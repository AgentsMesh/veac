use crate::{Apply, ApplyTarget, Clip, RelationItem, Sequence, TimeRange, Track, TrackKind};

pub struct SequenceActivity<'a> {
    sequence: &'a Sequence,
    has_solo: bool,
}

impl<'a> SequenceActivity<'a> {
    pub fn new(sequence: &'a Sequence) -> Self {
        let has_solo = sequence
            .tracks
            .iter()
            .any(|track| track.state.enabled && track.state.solo);
        Self { sequence, has_solo }
    }

    pub fn track_live(&self, track: &Track) -> bool {
        track.state.enabled && (!self.has_solo || track.state.solo)
    }

    pub fn item_live(&self, track: &Track, clip: &Clip) -> bool {
        self.track_live(track) && clip.enabled
    }

    pub fn visual_typed(&self, track: &Track, _clip: &Clip) -> bool {
        matches!(
            track.kind,
            TrackKind::Video | TrackKind::Visual | TrackKind::Caption
        )
    }

    pub fn visual_live(&self, track: &Track, clip: &Clip) -> bool {
        self.item_live(track, clip) && self.visual_typed(track, clip)
    }

    pub fn audio_typed(&self, track: &Track, clip: &Clip) -> bool {
        match track.kind {
            TrackKind::Audio => true,
            TrackKind::Video => clip.audio.is_some(),
            TrackKind::Visual | TrackKind::Caption => false,
        }
    }

    pub fn audio_live(&self, track: &Track, clip: &Clip) -> bool {
        self.item_live(track, clip)
            && !track.state.muted
            && self.audio_typed(track, clip)
            && clip.audio.as_ref().is_none_or(|audio| !audio.muted)
    }

    pub fn track_has_live_audio(&self, track: &Track) -> bool {
        track.clips.iter().any(|clip| self.audio_live(track, clip))
    }

    pub fn live_apply_targets(&self, apply: &Apply) -> Vec<RelationItem<'a>> {
        if !apply.enabled {
            return Vec::new();
        }
        let windows = active_windows(apply);
        if windows.is_empty() {
            return Vec::new();
        }
        self.sequence
            .tracks
            .iter()
            .flat_map(|track| {
                track
                    .clips
                    .iter()
                    .enumerate()
                    .filter_map(|(position, clip)| {
                        (self.visual_live(track, clip)
                            && target_contains(self.sequence, &apply.target, track, clip)
                            && windows
                                .iter()
                                .any(|window| overlaps(clip.record_range, *window)))
                        .then_some(RelationItem {
                            sequence: self.sequence,
                            track,
                            clip,
                            position,
                        })
                    })
            })
            .collect()
    }

    pub fn apply_live(&self, apply: &Apply) -> bool {
        !self.live_apply_targets(apply).is_empty()
    }
}

fn active_windows(apply: &Apply) -> Vec<TimeRange> {
    apply
        .stages
        .iter()
        .filter(|stage| stage.is_executable())
        .filter_map(|stage| match stage.active_range {
            None => Some(apply.record_range),
            Some(relative) => Some(TimeRange {
                start: apply.record_range.start.checked_add(relative.start).ok()?,
                duration: relative.duration,
            }),
        })
        .collect()
}

fn target_contains(sequence: &Sequence, target: &ApplyTarget, track: &Track, clip: &Clip) -> bool {
    match target {
        ApplyTarget::Layer { track_id } => track.id == *track_id,
        ApplyTarget::ItemSet { item_ids } => item_ids.contains(&clip.id),
        ApplyTarget::CompositeBand {
            from_track_id,
            through_track_id,
        } => {
            let from = sequence
                .tracks
                .iter()
                .find(|value| value.id == *from_track_id);
            let through = sequence
                .tracks
                .iter()
                .find(|value| value.id == *through_track_id);
            from.zip(through)
                .is_some_and(|(from, through)| (from.order..=through.order).contains(&track.order))
        }
    }
}

fn overlaps(left: TimeRange, right: TimeRange) -> bool {
    left.end()
        .ok()
        .zip(right.end().ok())
        .is_some_and(|(a, b)| left.start < b && right.start < a)
}
