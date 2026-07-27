use crate::edit::ChangeSet;
use crate::test_support::{sample_project, time};

use super::*;

#[test]
fn split_rejects_an_overflowed_clip_range_before_mutation() {
    let mut project = sample_project().project;
    let maximum = crate::MAX_SAFE_INTEGER as i64;
    let clip = &mut project.sequences[0].tracks[0].clips[0];
    clip.record_range.start = time(maximum);
    clip.record_range.duration = time(1);
    let before = project.clone();
    let mut changed = ChangeSet::new();
    let error = split_one(
        &mut project,
        &ItemId::new("itm_video").unwrap(),
        time(maximum),
        &ItemId::new("itm_right").unwrap(),
        &[],
        &mut changed,
    )
    .unwrap_err();
    assert!(error.message.contains("clip range is invalid"));
    assert_eq!(project, before);
    assert!(changed.is_empty());
}

#[test]
fn split_accepts_non_media_clips_without_source_mappings() {
    let mut project = sample_project().project;
    let mut changed = ChangeSet::new();
    split_one(
        &mut project,
        &ItemId::new("itm_caption").unwrap(),
        time(150),
        &ItemId::new("itm_caption_right").unwrap(),
        &[],
        &mut changed,
    )
    .unwrap();
    let clips = &project.sequences[0].tracks[1].clips;
    assert_eq!(clips.len(), 2);
    assert!(clips.iter().all(|clip| clip.source_mapping.is_none()));
    assert_eq!(clips[0].record_range.duration, time(150));
    assert_eq!(clips[1].record_range.start, time(150));
}
