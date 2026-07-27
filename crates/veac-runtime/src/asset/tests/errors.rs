use serde_json::{json, Value};

use super::*;

fn first_stream(mut value: Value, update: impl FnOnce(&mut Value)) -> Value {
    update(&mut value["streams"][0]);
    value
}

fn assert_field_error(value: Value, field: &'static str) {
    let error = parse(&value, auto_stream_intent()).unwrap_err();
    assert!(
        matches!(error, ProbeError::InvalidField { field: actual, .. } if actual == field),
        "unexpected error: {error:?}"
    );
    assert!(error.to_string().contains(field));
}

#[test]
fn rejects_missing_duplicate_and_unknown_stream_identity_fields() {
    assert_field_error(
        first_stream(complete_output(), |stream| stream["index"] = Value::Null),
        "streams[].index",
    );
    let mut duplicate = complete_output();
    let stream = duplicate["streams"][0].clone();
    duplicate["streams"].as_array_mut().unwrap().push(stream);
    assert_field_error(duplicate, "streams[].index");
    assert_field_error(
        first_stream(complete_output(), |stream| {
            stream["codec_type"] = json!("telepathy")
        }),
        "streams[].codec_type",
    );
    assert_field_error(
        first_stream(complete_output(), |stream| {
            stream["codec_type"] = Value::Null
        }),
        "streams[].codec_type",
    );
    assert_field_error(
        first_stream(complete_output(), |stream| stream["codec_name"] = json!("")),
        "streams[].codec_name",
    );
}

#[test]
fn rejects_missing_or_unsafe_container_format_names() {
    for value in [Value::Null, json!(""), json!("mov,MP4"), json!("mov,,mp4")] {
        let mut output = complete_output();
        output["format"]["format_name"] = value;
        assert_field_error(output, "format.format_name");
    }
}

#[test]
fn rejects_invalid_dispositions_and_typed_stream_facts() {
    assert_field_error(
        first_stream(complete_output(), |stream| {
            stream["disposition"]["default"] = json!(2)
        }),
        "streams[].disposition.default",
    );
    assert_field_error(
        first_stream(complete_output(), |stream| {
            stream["disposition"]["attached_pic"] = json!(1)
        }),
        "streams[].disposition",
    );
    for field in ["sample_rate", "channel_layout"] {
        assert_field_error(
            first_stream(complete_output(), |stream| stream[field] = json!("")),
            if field == "sample_rate" {
                "streams[].sample_rate"
            } else {
                "streams[].channel_layout"
            },
        );
    }
    assert_field_error(
        first_stream(complete_output(), |stream| stream["channels"] = json!(300)),
        "streams[].channels",
    );
    assert_field_error(
        first_stream(complete_output(), |stream| stream["channels"] = Value::Null),
        "streams[].channels",
    );
    let video_index = 1;
    for field in ["width", "height"] {
        let mut value = complete_output();
        value["streams"][video_index][field] = json!(0);
        assert_field_error(
            value,
            if field == "width" {
                "streams[].width"
            } else {
                "streams[].height"
            },
        );
    }
}

#[test]
fn rejects_invalid_exact_times_as_structured_field_errors() {
    let cases = [
        ("start_time", "-0.1", "streams[].start_time"),
        ("duration", "0", "streams[].duration"),
        ("duration", "oops", "streams[].duration"),
        ("duration", ".", "streams[].duration"),
        ("duration", "0.0000000001", "streams[].duration"),
        ("duration", "9007199254740992", "streams[].duration"),
        (
            "duration",
            "999999999999999999999999999999999999999999999999999999999999",
            "streams[].duration",
        ),
        (
            "duration",
            "170141183460469231731687303715884105727.1",
            "streams[].duration",
        ),
        (
            "duration",
            "17014118346046923173168730371588410572.9",
            "streams[].duration",
        ),
        ("duration", "9223372036854775808", "streams[].duration"),
    ];
    for (field, raw, expected) in cases {
        assert_field_error(
            first_stream(complete_output(), |stream| stream[field] = json!(raw)),
            expected,
        );
    }
    let mut zero_container = complete_output();
    zero_container["format"]["duration"] = json!("0.0");
    assert_field_error(zero_container, "format.duration");
}

#[test]
fn rejects_invalid_aspect_ratio_and_rotation() {
    let video_index = 1;
    for ratio in ["invalid", "x:1", "1:x", "0:1", "1:0"] {
        let mut value = complete_output();
        value["streams"][video_index]["sample_aspect_ratio"] = json!(ratio);
        assert_field_error(value, "sample_aspect_ratio");
    }
    for rotation in [json!(12.5), json!(40000)] {
        let mut value = complete_output();
        value["streams"][video_index]["side_data_list"][0]["rotation"] = rotation;
        assert_field_error(value, "streams[].rotation");
    }
}

#[test]
fn normalizes_data_and_attachment_stream_types() {
    let value = json!({ "format": { "format_name": "data" }, "streams": [
        { "index": 8, "codec_type": "attachment", "codec_name": "ttf" },
        { "index": 7, "codec_type": "data", "codec_name": "bin_data" }
    ] });
    let snapshot = parse(&value, auto_stream_intent()).unwrap();
    assert_eq!(
        snapshot.streams[0].media_type,
        veac_ir::ProbedStreamType::Data
    );
    assert_eq!(
        snapshot.streams[1].media_type,
        veac_ir::ProbedStreamType::Attachment
    );
}
