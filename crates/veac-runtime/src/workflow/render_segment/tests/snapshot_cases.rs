use veac_ir::{ProbedStreamType, Rational, RationalTime, StreamSelection};

use super::support::*;
use super::*;

#[test]
fn exact_silent_and_audio_snapshots_are_accepted() {
    for with_audio in [false, true] {
        let contract = contract(with_audio);
        let record = record(&contract);
        validate_full_render_segment_snapshot(&contract, &record, &snapshot(&contract, &record))
            .unwrap();
    }
}

#[test]
fn record_and_probe_identity_are_authoritative() {
    let contract = contract(false);
    let mut invalid_record = record(&contract);
    let mut probe = snapshot(&contract, &invalid_record);
    invalid_record.key.value = "ab".repeat(32);
    assert_kind(
        &contract,
        &invalid_record,
        &probe,
        WorkflowErrorKind::InvalidContract,
    );

    let record = record(&contract);
    probe.observed_identity.digest = "cd".repeat(32);
    assert_kind(
        &contract,
        &record,
        &probe,
        WorkflowErrorKind::SourceIdentityMismatch,
    );
    let mut empty = record.clone();
    empty.size_bytes = 0;
    assert_kind(
        &contract,
        &empty,
        &snapshot(&contract, &record),
        WorkflowErrorKind::InvalidContract,
    );
}

#[test]
fn exact_container_and_stream_inventory_are_required() {
    let contract = contract(false);
    let record = record(&contract);
    let mut probe = snapshot(&contract, &record);
    probe.container_brand = Some("qt  ".to_owned());
    assert_tool_failure(&contract, &record, &probe);

    let mut probe = snapshot(&contract, &record);
    probe.streams.push(extra_stream());
    assert_tool_failure(&contract, &record, &probe);

    let mut probe = snapshot(&contract, &record);
    probe.selected_video_stream = Some(StreamSelection {
        global_index: 99,
        type_index: 0,
    });
    assert_tool_failure(&contract, &record, &probe);
}

#[test]
fn video_codec_geometry_pixel_clock_and_orientation_are_exact() {
    let contract = contract(false);
    let record = record(&contract);
    for mutation in 0..8 {
        let mut probe = snapshot(&contract, &record);
        let stream = &mut probe.streams[0];
        let video = stream.video.as_mut().unwrap();
        match mutation {
            0 => stream.codec = "hevc".to_owned(),
            1 => video.width -= 2,
            2 => video.frame_rate = Some(Rational::new(25, 1).unwrap()),
            3 => video.pixel_format = "yuv420p10le".to_owned(),
            4 => video.sample_aspect_ratio = Rational::new(2, 1).unwrap(),
            5 => video.rotation_degrees = 90,
            6 => stream.start_time = None,
            7 => stream.time_base = None,
            _ => unreachable!(),
        }
        assert_tool_failure(&contract, &record, &probe);
    }
}

#[test]
fn duration_is_bounded_on_both_stream_and_container() {
    let contract = contract(false);
    let record = record(&contract);
    let mut probe = snapshot(&contract, &record);
    probe.streams[0].duration = Some(RationalTime::new(2, 1).unwrap());
    assert_tool_failure(&contract, &record, &probe);

    let mut probe = snapshot(&contract, &record);
    probe.container_duration = Some(RationalTime::new(2, 1).unwrap());
    assert_tool_failure(&contract, &record, &probe);
}

#[test]
fn audio_presence_codec_format_origin_and_duration_are_exact() {
    let contract = contract(true);
    let record = record(&contract);
    for mutation in 0..6 {
        let mut probe = snapshot(&contract, &record);
        let stream = &mut probe.streams[1];
        match mutation {
            0 => stream.codec = "opus".to_owned(),
            1 => stream.audio.as_mut().unwrap().sample_rate = 44_100,
            2 => stream.audio.as_mut().unwrap().channels = 1,
            3 => stream.start_time = None,
            4 => stream.duration = Some(RationalTime::new(2, 1).unwrap()),
            5 => stream.media_type = ProbedStreamType::Data,
            _ => unreachable!(),
        }
        assert_tool_failure(&contract, &record, &probe);
    }
}

fn assert_tool_failure(
    contract: &veac_artifact::FullRenderSegmentContract,
    record: &veac_artifact::ArtifactRecord,
    probe: &veac_ir::MediaProbeSnapshot,
) {
    assert_kind(contract, record, probe, WorkflowErrorKind::ToolFailure);
}

fn assert_kind(
    contract: &veac_artifact::FullRenderSegmentContract,
    record: &veac_artifact::ArtifactRecord,
    probe: &veac_ir::MediaProbeSnapshot,
    expected: WorkflowErrorKind,
) {
    let error = validate_full_render_segment_snapshot(contract, record, probe).unwrap_err();
    assert_eq!(error.kind, expected);
}
