use veac_ir::{ApplyTarget, ItemId, TimeRange, TrackKind};

use crate::{ResolvedApplyItem, ResolvedApplyTarget, ResolvedTrack};

use super::apply_ranges;

pub(super) fn resolve(
    authored: &ApplyTarget,
    range: TimeRange,
    tracks: &[ResolvedTrack],
) -> Option<ResolvedApplyTarget> {
    match authored {
        ApplyTarget::CompositeBand {
            from_track_id,
            through_track_id,
        } => {
            let from = position(tracks, from_track_id)?;
            let through = position(tracks, through_track_id)?;
            let selected: Vec<_> = tracks[from..=through]
                .iter()
                .filter(|track| visual_track(track))
                .collect();
            let active_ranges = ranges(selected.iter().flat_map(|track| &track.clips), range);
            (!active_ranges.is_empty()).then(|| ResolvedApplyTarget::CompositeBand {
                from_track_id: from_track_id.clone(),
                through_track_id: through_track_id.clone(),
                track_ids: selected.iter().map(|track| track.id.clone()).collect(),
                active_ranges,
            })
        }
        ApplyTarget::Layer { track_id } => {
            let track = tracks.iter().find(|track| track.id == *track_id)?;
            if !visual_track(track) {
                return None;
            }
            let clips: Vec<_> = track
                .clips
                .iter()
                .filter(|clip| clip.visual.is_some())
                .collect();
            let active_ranges = ranges(clips.iter().copied(), range);
            (!active_ranges.is_empty()).then(|| ResolvedApplyTarget::Layer {
                track_id: track_id.clone(),
                item_ids: clips.iter().map(|clip| clip.id.clone()).collect(),
                active_ranges,
            })
        }
        ApplyTarget::ItemSet { item_ids } => {
            let items: Vec<_> = item_ids
                .iter()
                .filter_map(|item_id| item(tracks, item_id, range))
                .collect();
            (!items.is_empty()).then_some(ResolvedApplyTarget::ItemSet { items })
        }
    }
}

fn position(tracks: &[ResolvedTrack], id: &veac_ir::TrackId) -> Option<usize> {
    tracks.iter().position(|track| track.id == *id)
}

fn visual_track(track: &ResolvedTrack) -> bool {
    track.state.visual_enabled
        && matches!(
            track.kind,
            TrackKind::Video | TrackKind::Visual | TrackKind::Caption
        )
}

fn item(tracks: &[ResolvedTrack], id: &ItemId, range: TimeRange) -> Option<ResolvedApplyItem> {
    tracks.iter().find_map(|track| {
        if !visual_track(track) {
            return None;
        }
        let clip = track
            .clips
            .iter()
            .find(|clip| clip.id == *id && clip.visual.is_some())?;
        Some(ResolvedApplyItem {
            track_id: track.id.clone(),
            item_id: id.clone(),
            active_range: apply_ranges::intersect(range, clip.record_range)?,
        })
    })
}

fn ranges<'a>(
    clips: impl Iterator<Item = &'a crate::ResolvedClip>,
    range: TimeRange,
) -> Vec<TimeRange> {
    apply_ranges::merge(
        clips
            .filter_map(|clip| apply_ranges::intersect(range, clip.record_range))
            .collect(),
    )
}
