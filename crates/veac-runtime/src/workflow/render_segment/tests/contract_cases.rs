use veac_ir::{ProbedStreamType, StreamSelection};

use super::support::*;
use super::*;

#[test]
fn malformed_record_digests_keep_contract_sources() {
    let segment = contract(false);
    let valid = record(&segment);
    for content in [false, true] {
        let mut invalid = valid.clone();
        if content {
            invalid.content.value = "bad".to_owned();
        } else {
            invalid.key.value = "bad".to_owned();
        }
        let error = contract::validate_record(&segment, &invalid, 1_000).unwrap_err();
        assert_eq!(error.kind, WorkflowErrorKind::InvalidContract);
        assert!(std::error::Error::source(&error).is_some());
    }
}

#[test]
fn normalized_snapshot_header_selection_and_silent_audio_are_exact() {
    let segment = contract(false);
    let record = record(&segment);
    for mutate in 0..4 {
        let mut probe = snapshot(&segment, &record);
        match mutate {
            0 => probe.schema_version += 1,
            1 => probe.selection_policy = "future-policy".to_owned(),
            2 => probe.selected_video_stream = None,
            3 => {
                probe.selected_audio_stream = Some(StreamSelection {
                    global_index: 0,
                    type_index: 0,
                })
            }
            _ => unreachable!(),
        }
        assert_tool_failure(&segment, &record, &probe);
    }

    let mut inconsistent = snapshot(&segment, &record);
    inconsistent.streams[0].media_type = ProbedStreamType::Audio;
    assert_tool_failure(&segment, &record, &inconsistent);
}

#[test]
fn missing_video_and_audio_facts_or_durations_are_rejected() {
    let silent = contract(false);
    let silent_record = record(&silent);
    let mut no_video = snapshot(&silent, &silent_record);
    no_video.streams[0].video = None;
    assert_tool_failure(&silent, &silent_record, &no_video);

    let mut no_video_duration = snapshot(&silent, &silent_record);
    no_video_duration.streams[0].duration = None;
    no_video_duration.container_duration = None;
    assert_tool_failure(&silent, &silent_record, &no_video_duration);

    let audible = contract(true);
    let audible_record = record(&audible);
    let mut no_audio = snapshot(&audible, &audible_record);
    no_audio.streams[1].audio = None;
    assert_tool_failure(&audible, &audible_record, &no_audio);

    let mut no_audio_duration = snapshot(&audible, &audible_record);
    no_audio_duration.streams[1].duration = None;
    no_audio_duration.container_duration = None;
    assert_tool_failure(&audible, &audible_record, &no_audio_duration);
}

fn assert_tool_failure(
    segment: &veac_artifact::FullRenderSegmentContract,
    record: &veac_artifact::ArtifactRecord,
    probe: &veac_ir::MediaProbeSnapshot,
) {
    let error = validate_full_render_segment_snapshot(segment, record, probe).unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ToolFailure);
}
