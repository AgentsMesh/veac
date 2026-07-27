use crate::reject_duplicate_json_keys;

#[test]
fn strict_json_walks_every_scalar_sequence_and_map_shape() {
    for json in [
        "true",
        "-9223372036854775808",
        "18446744073709551615",
        "1.25e3",
        r#""text""#,
        "null",
        r#"[true,-1,2,3.5,"x",null,{},[]]"#,
        r#"{"outer":{"value":1},"items":[{"value":2},{"value":3}]}"#,
    ] {
        reject_duplicate_json_keys(json).unwrap();
    }
}

#[test]
fn duplicate_detection_is_recursive_and_decodes_escaped_names() {
    for json in [
        r#"{"outer":{"same":1,"same":2}}"#,
        r#"[{"same":1,"same":2}]"#,
        r#"{"a":1,"\u0061":2}"#,
    ] {
        let error = reject_duplicate_json_keys(json).unwrap_err();
        assert!(error.to_string().contains("duplicate object name"));
    }
    reject_duplicate_json_keys(r#"{"left":{"same":1},"right":{"same":2}}"#).unwrap();
}

#[test]
fn strict_json_rejects_trailing_values_and_malformed_map_entries() {
    for json in ["{} {}", r#"{"value":}"#, r#"{"value":1"#, "1 2"] {
        assert!(reject_duplicate_json_keys(json).is_err(), "accepted {json}");
    }
}
