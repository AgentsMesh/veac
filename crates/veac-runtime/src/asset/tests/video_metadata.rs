use serde_json::{json, Value};

use super::*;

#[test]
fn normalizes_unknown_profile_level_and_container_brand_sentinels() {
    let mut value = complete_output();
    value["format"]["tags"]["major_brand"] = json!("N/A");
    value["streams"][1]["profile"] = json!("unknown");
    value["streams"][1]["level"] = json!(-99);
    let probe = parse(&value, auto_stream_intent()).unwrap();
    let video = probe.streams[0].video.as_ref().unwrap();
    assert!(probe.container_brand.is_none());
    assert!(video.profile.is_none());
    assert!(video.level.is_none());
}

#[test]
fn rejects_missing_or_oversized_pixel_and_profile_facts() {
    for (field, value, expected) in [
        ("pix_fmt", Value::Null, "streams[].pix_fmt"),
        ("pix_fmt", json!("p".repeat(65)), "streams[].pix_fmt"),
        ("profile", json!("p".repeat(129)), "streams[].profile"),
        ("profile", json!("bad\nprofile"), "streams[].profile"),
        ("level", json!(-2), "streams[].level"),
    ] {
        let mut output = complete_output();
        output["streams"][1][field] = value;
        assert_field(output, expected);
    }
}

#[test]
fn rejects_noncanonical_container_brands() {
    for brand in ["x".repeat(65), "bad\nbrand".to_owned()] {
        let mut output = complete_output();
        output["format"]["tags"]["major_brand"] = json!(brand);
        assert_field(output, "format.tags.major_brand");
    }
}

fn assert_field(value: Value, expected: &'static str) {
    let error = parse(&value, auto_stream_intent()).unwrap_err();
    assert!(
        matches!(error, ProbeError::InvalidField { field, .. } if field == expected),
        "{error:?}"
    );
}
