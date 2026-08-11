use serde_json::{json, Value};

use super::*;

#[test]
fn rejects_probe_json_byte_shape_stream_and_engine_budgets() {
    let too_large = " ".repeat(veac_artifact::MAX_ARTIFACT_METADATA_BYTES as usize + 1);
    assert!(matches!(
        parse_raw(&too_large),
        Err(ProbeError::ResourceLimit { .. })
    ));

    let mut nested = Value::Null;
    for _ in 0..=veac_artifact::MAX_ARTIFACT_JSON_DEPTH {
        nested = json!([nested]);
    }
    let shaped = json!({"format":{"format_name":"data"},"extra":nested});
    assert!(matches!(
        parse_raw(&shaped.to_string()),
        Err(ProbeError::ResourceLimit { .. })
    ));

    let streams = (0..4_097)
        .map(|index| json!({"index":index}))
        .collect::<Vec<_>>();
    let inventory = json!({"format":{"format_name":"data"},"streams":streams});
    assert!(matches!(
        parse_raw(&inventory.to_string()),
        Err(ProbeError::ResourceLimit { .. })
    ));

    let error = parse_ffprobe_json(
        r#"{"format":{"format_name":"data"}}"#,
        identity(),
        auto_stream_intent(),
        &"x".repeat(1_025),
    )
    .unwrap_err();
    assert!(matches!(error, ProbeError::ResourceLimit { .. }));
}

#[test]
fn playable_streams_require_timing_while_auxiliary_video_does_not() {
    let mut value = complete_output();
    value["streams"][1]["time_base"] = Value::Null;
    assert_timing_error(&value);

    let mut value = complete_output();
    value["streams"][1]["avg_frame_rate"] = Value::Null;
    value["streams"][1]["r_frame_rate"] = Value::Null;
    assert_timing_error(&value);

    let mut value = complete_output();
    value["streams"][0]["time_base"] = Value::Null;
    assert_timing_error(&value);

    let mut value = complete_output();
    value["streams"][1]["time_base"] = Value::Null;
    value["streams"][1]["avg_frame_rate"] = Value::Null;
    value["streams"][1]["r_frame_rate"] = Value::Null;
    value["streams"][1]["disposition"]["attached_pic"] = json!(1);
    assert!(parse(&value, auto_stream_intent()).is_ok());
}

#[test]
fn absurd_auxiliary_frame_rates_normalize_but_playable_video_fails_closed() {
    let mut playable = complete_output();
    playable["streams"][1]["avg_frame_rate"] = json!("90000/1");
    playable["streams"][1]["r_frame_rate"] = json!("90000/1");
    assert_field(&playable, "streams[].frame_rate");

    for disposition in ["attached_pic", "timed_thumbnails"] {
        let mut auxiliary = playable.clone();
        auxiliary["streams"][1]["disposition"][disposition] = json!(1);
        let snapshot = parse(&auxiliary, auto_stream_intent()).unwrap();
        assert!(snapshot
            .streams
            .iter()
            .find(|stream| stream.global_index == 2)
            .unwrap()
            .video
            .as_ref()
            .unwrap()
            .frame_rate
            .is_none());
    }
}

#[test]
fn average_and_declared_rates_do_not_claim_cadence_without_packet_evidence() {
    let mut variable = complete_output();
    variable["streams"][1]["avg_frame_rate"] = json!("4/1");
    variable["streams"][1]["r_frame_rate"] = json!("10/1");
    let snapshot = parse(&variable, auto_stream_intent()).unwrap();
    let info = snapshot.streams[0].video.as_ref().unwrap();
    assert_eq!(info.frame_rate, Some(veac_ir::Rational::new(4, 1).unwrap()));
    assert_eq!(info.cadence, veac_ir::VideoCadence::Unknown);

    variable["streams"][1]["avg_frame_rate"] = Value::Null;
    let snapshot = parse(&variable, auto_stream_intent()).unwrap();
    let info = snapshot.streams[0].video.as_ref().unwrap();
    assert_eq!(
        info.frame_rate,
        Some(veac_ir::Rational::new(10, 1).unwrap())
    );
    assert_eq!(info.cadence, veac_ir::VideoCadence::Unknown);
}

#[test]
fn average_rate_precedes_strict_declared_rate_fallback() {
    let mut value = complete_output();
    value["streams"][1]["avg_frame_rate"] = json!("30/1");
    value["streams"][1]["r_frame_rate"] = json!("not-a-rational");
    let snapshot = parse(&value, auto_stream_intent()).unwrap();
    assert_eq!(
        snapshot.streams[0].video.as_ref().unwrap().frame_rate,
        Some(veac_ir::Rational::new(30, 1).unwrap())
    );

    value["streams"][1]["avg_frame_rate"] = json!("90000/1");
    value["streams"][1]["r_frame_rate"] = json!("24/1");
    assert_field(&value, "streams[].frame_rate");

    value["streams"][1]["avg_frame_rate"] = Value::Null;
    let snapshot = parse(&value, auto_stream_intent()).unwrap();
    assert_eq!(
        snapshot.streams[0].video.as_ref().unwrap().frame_rate,
        Some(veac_ir::Rational::new(24, 1).unwrap())
    );
}

#[test]
fn codec_and_channel_layout_are_bounded_control_fields() {
    let mut value = complete_output();
    value["streams"][1]["codec_name"] = json!("x".repeat(129));
    assert_field(&value, "streams[].codec_name");

    let mut value = complete_output();
    value["streams"][0]["channel_layout"] =
        json!("x".repeat(veac_ir::MAX_CHANNEL_LAYOUT_BYTES + 1));
    assert_field(&value, "streams[].channel_layout");

    let mut value = complete_output();
    value["streams"][0]
        .as_object_mut()
        .unwrap()
        .remove("channel_layout");
    let snapshot = parse(&value, auto_stream_intent()).unwrap();
    assert_eq!(
        snapshot.streams[1].audio.as_ref().unwrap().channel_layout,
        "unknown"
    );
}

fn parse_raw(json: &str) -> Result<MediaProbeSnapshot, ProbeError> {
    parse_ffprobe_json(json, identity(), auto_stream_intent(), FIXTURE_PROBE_ENGINE)
}

fn assert_timing_error(value: &Value) {
    assert_field(value, "streams[].timing");
}

fn assert_field(value: &Value, expected: &'static str) {
    let error = parse(value, auto_stream_intent()).unwrap_err();
    assert!(matches!(error, ProbeError::InvalidField { field, .. } if field == expected));
}
