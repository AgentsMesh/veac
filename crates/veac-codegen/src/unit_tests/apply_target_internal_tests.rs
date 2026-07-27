use super::*;
use crate::emitter::preflight::Check;
use crate::unit_tests::emitter_tests::composition_advanced::advanced_plan;
use crate::unit_tests::emitter_tests::support::time;
use veac_plan::canonical::{TimeRange, TrackKind};
use veac_plan::{ResolvedApplyItem, ResolvedApplyTarget, ResolvedClip, ResolvedTrack};

fn intersection_range(
    apply: &ResolvedApply,
    track: &ResolvedTrack,
    clip: &ResolvedClip,
) -> TimeRange {
    intersection(apply.record_range, clip.record_range)
        .unwrap_or_else(|| panic!("{} and {} must overlap", apply.id, track.id))
}

#[test]
fn validates_sorted_item_sets_across_visual_tracks() {
    let plan = advanced_plan();
    let sequence = &plan.sequences[0];
    let apply = &sequence.applies[0];
    let mut items = sequence
        .tracks
        .iter()
        .map(|track| {
            let clip = &track.clips[0];
            ResolvedApplyItem {
                track_id: track.id.clone(),
                item_id: clip.id.clone(),
                active_range: intersection_range(apply, track, clip),
            }
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.item_id.cmp(&right.item_id));

    let mut item_apply = apply.clone();
    item_apply.target = ResolvedApplyTarget::ItemSet { items };
    let mut check = Check::default();
    validate(&mut check, sequence, &item_apply);
    assert!(contains_item(
        &item_apply,
        &sequence.tracks[0].id,
        &sequence.tracks[0].clips[0].id
    ));
}

#[test]
fn helper_contracts_reject_invalid_bands_ranges_and_tracks() {
    let plan = advanced_plan();
    let sequence = &plan.sequences[0];
    let apply = &sequence.applies[0];
    let first = &sequence.tracks[0];
    let second = &sequence.tracks[1];

    let first_item = first.clips[0].id.clone();
    let active = [apply.record_range];
    assert!(layer_valid(
        sequence,
        &first.id,
        std::slice::from_ref(&first_item),
        &active,
        apply.record_range
    ));
    assert!(!layer_valid(
        sequence,
        &second.id,
        std::slice::from_ref(&first_item),
        &active,
        apply.record_range
    ));
    assert!(band_valid(
        sequence,
        &first.id,
        &second.id,
        &[first.id.clone(), second.id.clone()],
        &active,
        apply.record_range
    ));
    assert!(!band_valid(
        sequence,
        &second.id,
        &first.id,
        &[first.id.clone(), second.id.clone()],
        &active,
        apply.record_range
    ));

    let ranges = [
        TimeRange::new(apply.record_range.start, time(60)).expect("first range"),
        TimeRange::new(time(300), time(60)).expect("second range"),
    ];
    assert!(ranges_valid(&ranges, apply.record_range));
    assert!(!ranges_valid(
        &[apply.record_range, apply.record_range],
        apply.record_range
    ));
    assert!(within(apply.record_range, ranges[0]));
    assert!(!within(
        apply.record_range,
        TimeRange::new(time(0), time(60)).expect("outside range")
    ));
    assert!(visual_track(TrackKind::Video));
    assert!(visual_track(TrackKind::Visual));
    assert!(!visual_track(TrackKind::Audio));

    assert_eq!(intersection(apply.record_range, ranges[0]), Some(ranges[0]));
}
