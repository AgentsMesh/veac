use super::support;

#[test]
fn explicit_microsecond_timebase_preserves_exact_clip_ticks() {
    let item = support::solid("flash", "#ffffffff", "1ms", "1us");
    let source = support::visual_project_with_timebase(&[&item], 3_000_000);
    let envelope = support::envelope(&source);
    assert_eq!(envelope.project.timebase, 3_000_000);
    let range = support::clip(&envelope, 0).record_range;
    assert_eq!(range.start.value, 3_000);
    assert_eq!(range.duration.value, 3);
    assert_eq!(range.start.timescale, 3_000_000);
    assert_eq!(range.duration.timescale, 3_000_000);
}

#[test]
fn clip_sorting_is_canonical_by_start() {
    let late = support::solid("late", "#ffffffff", "2s", "1s");
    let early = support::solid("early", "#000000ff", "0s", "1s");
    let envelope = support::envelope(&support::visual_project(&[&late, &early]));
    let clips = &envelope.project.sequences[0].tracks[0].clips;
    assert_eq!(clips[0].record_range.start.value, 0);
    assert_eq!(clips[1].record_range.start.value, 1_200);
}

#[test]
fn equal_start_clips_preserve_authored_attachment_order() {
    let first = support::solid("first", "#ff0000ff", "0s", "1s");
    let second = support::solid("second", "#0000ffff", "0s", "1s");
    let forward = support::envelope(&support::visual_project(&[&first, &second]));
    let reverse = support::envelope(&support::visual_project(&[&second, &first]));
    let ids = |value: &veac_ir::ProjectEnvelope| {
        value.project.sequences[0].tracks[0]
            .clips
            .iter()
            .map(|clip| clip.id.to_string())
            .collect::<Vec<_>>()
    };
    let forward = ids(&forward);
    let reverse = ids(&reverse);
    assert_eq!(forward, reverse.iter().rev().cloned().collect::<Vec<_>>());
}

#[test]
fn individually_safe_ticks_cannot_overflow_a_time_range() {
    let item = support::solid("overflow", "#ffffffff", "9223372036854775807s", "1s");
    let source = support::visual_project_with_timebase(&[&item], 1);
    let error = support::error(&source);
    assert_eq!(error.code, "PROGRAM_EXECUTABLE_LOWER");
    assert!(error.message.contains("EXECUTABLE_LOWER_TIME"));
}
