use veac_ir::ProbedStreamType;

use super::super::test_support::*;
use super::*;

#[test]
fn every_ffmpeg_artifact_snapshot_satisfies_its_typed_contract() {
    for (spec, probe) in valid_cases() {
        let result = crate::workflow::validate_media_artifact_snapshot(&probe, &spec);
        assert!(result.is_ok(), "spec={spec:?} result={result:?}");
    }
    let (analysis, probe) = analysis_case();
    assert!(validate(&probe, &analysis).is_err());
}

#[test]
fn stream_set_selection_and_video_contract_failures_are_distinct() {
    let (spec, probe) = valid_cases().remove(0);
    let mut extra = probe.clone();
    extra.streams.push(extra.streams[0].clone());
    assert_error(&extra, &spec, "stream set");

    let mut missing = probe.clone();
    missing.selected_video_stream = None;
    assert_error(&missing, &spec, "selected");
    let mut inconsistent = probe.clone();
    inconsistent
        .selected_video_stream
        .as_mut()
        .unwrap()
        .global_index = 99;
    assert_error(&inconsistent, &spec, "inconsistent");

    let mut no_facts = probe.clone();
    no_facts.streams[0].video = None;
    assert_error(&no_facts, &spec, "video facts");
    let mut mismatch = probe.clone();
    mismatch.streams[0].codec = "hevc".into();
    assert_error(&mismatch, &spec, "video does not match");
    let mut no_clock = probe.clone();
    no_clock.streams[0].time_base = None;
    assert_error(&no_clock, &spec, "video does not match");
    let mut origin = probe.clone();
    origin.streams[0].start_time = Some(time(1));
    assert_error(&origin, &spec, "start at zero");
    let mut duration = probe.clone();
    duration.streams[0].duration = None;
    assert_error(&duration, &spec, "duration");

    assert!(video(&probe, 320, 180, "h264", None, Some(time(100)))
        .unwrap_err()
        .to_string()
        .contains("frame rate"));
    assert!(selected(&probe, ProbedStreamType::Data).is_err());
}

#[test]
fn audio_contract_rejects_missing_facts_format_origin_and_duration() {
    let (spec, probe) = valid_cases().remove(1);
    let mut no_facts = probe.clone();
    no_facts.streams[0].audio = None;
    assert_error(&no_facts, &spec, "audio facts");
    let mut mismatch = probe.clone();
    mismatch.streams[0].codec = "aac".into();
    assert_error(&mismatch, &spec, "audio does not match");
    let mut origin = probe.clone();
    origin.streams[0].start_time = Some(time(1));
    assert_error(&origin, &spec, "start at zero");
    let mut duration = probe;
    duration.streams[0].duration = None;
    assert_error(&duration, &spec, "duration");
}

fn assert_error(probe: &veac_ir::MediaProbeSnapshot, spec: &MediaArtifactSpec, message: &str) {
    let error = validate(probe, spec).unwrap_err();
    assert!(error.to_string().contains(message), "{error:?}");
}
