use super::*;

#[test]
fn optional_time_rejects_sign_precision_and_numeric_overflow() {
    for raw in [
        "-1",
        "1.1234567891",
        "170141183460469231731687303715884105728",
        "9223372036854775808",
        "-",
    ] {
        assert!(
            optional_time("duration", Some(raw), true).is_err(),
            "accepted {raw}"
        );
    }
    assert_eq!(optional_time("duration", Some("N/A"), true).unwrap(), None);
}

#[test]
fn aspect_and_frame_ratios_reject_malformed_numbers() {
    for raw in ["1", "x:1", "1:x", "1:0", "-1:1"] {
        assert!(sample_aspect_ratio(Some(raw)).is_err());
    }
    for raw in [None, Some("1"), Some("x/1"), Some("1/x"), Some("1/0")] {
        assert!(positive_ratio("rate", raw).is_err());
    }
    assert_eq!(optional_positive_ratio("rate", Some("0/0")).unwrap(), None);
}
