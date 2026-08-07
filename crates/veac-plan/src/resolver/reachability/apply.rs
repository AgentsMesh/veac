use veac_ir::{Apply, ApplyStage, ApplyTarget, ItemId, Sequence, TimeRange, Track};

use super::ClipDemand;

pub(super) fn active(
    sequence: &Sequence,
    apply: &Apply,
    clip_demand: impl Fn(&ItemId) -> Option<ClipDemand> + Copy,
) -> bool {
    apply
        .stages
        .iter()
        .any(|stage| stage_active(sequence, apply, stage, clip_demand))
}

pub(super) fn stage_active(
    sequence: &Sequence,
    apply: &Apply,
    stage: &ApplyStage,
    clip_demand: impl Fn(&ItemId) -> Option<ClipDemand> + Copy,
) -> bool {
    if !apply.enabled || !stage.is_executable() {
        return false;
    }
    let Some(stage_range) =
        super::super::apply_ranges::absolute(apply.record_range, stage.active_range)
    else {
        return false;
    };
    active_ranges(sequence, apply, clip_demand)
        .iter()
        .any(|range| super::super::apply_ranges::intersect(*range, stage_range).is_some())
}

fn active_ranges(
    sequence: &Sequence,
    apply: &Apply,
    clip_demand: impl Fn(&ItemId) -> Option<ClipDemand> + Copy,
) -> Vec<TimeRange> {
    match &apply.target {
        ApplyTarget::Layer { track_id } => sequence
            .tracks
            .iter()
            .filter(|track| track.id == *track_id)
            .flat_map(|track| track_ranges(track, apply, clip_demand))
            .collect(),
        ApplyTarget::ItemSet { item_ids } => sequence
            .tracks
            .iter()
            .flat_map(|track| &track.clips)
            .filter(|clip| item_ids.contains(&clip.id))
            .filter_map(|clip| clip_range(clip, apply, clip_demand))
            .collect(),
        ApplyTarget::CompositeBand {
            from_track_id,
            through_track_id,
        } => band(sequence, from_track_id, through_track_id)
            .into_iter()
            .flat_map(|track| track_ranges(track, apply, clip_demand))
            .collect(),
    }
}

fn track_ranges(
    track: &Track,
    apply: &Apply,
    clip_demand: impl Fn(&ItemId) -> Option<ClipDemand> + Copy,
) -> Vec<TimeRange> {
    track
        .clips
        .iter()
        .filter_map(|clip| clip_range(clip, apply, clip_demand))
        .collect()
}

fn clip_range(
    clip: &veac_ir::Clip,
    apply: &Apply,
    clip_demand: impl Fn(&ItemId) -> Option<ClipDemand>,
) -> Option<TimeRange> {
    clip_demand(&clip.id)
        .is_some_and(|demand| demand.visual)
        .then(|| super::super::apply_ranges::intersect(apply.record_range, clip.record_range))
        .flatten()
}

fn band<'a>(
    sequence: &'a Sequence,
    from: &veac_ir::TrackId,
    through: &veac_ir::TrackId,
) -> Vec<&'a Track> {
    let tracks = sorted_tracks(sequence);
    let Some(start) = tracks.iter().position(|track| track.id == *from) else {
        return Vec::new();
    };
    let Some(end) = tracks.iter().position(|track| track.id == *through) else {
        return Vec::new();
    };
    if start <= end {
        tracks[start..=end].to_vec()
    } else {
        Vec::new()
    }
}

fn sorted_tracks(sequence: &Sequence) -> Vec<&Track> {
    let mut tracks: Vec<_> = sequence.tracks.iter().enumerate().collect();
    tracks.sort_by(|left, right| {
        (left.1.order, left.0, &left.1.id).cmp(&(right.1.order, right.0, &right.1.id))
    });
    tracks.into_iter().map(|(_, track)| track).collect()
}
