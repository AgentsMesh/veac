use super::decode_escape;

#[test]
fn incomplete_and_invalid_unicode_escapes_keep_specific_errors() {
    assert_eq!(
        decode_escape("").err().unwrap(),
        "escape sequence is incomplete"
    );
    assert_eq!(
        decode_escape("u{1").err().unwrap(),
        "Unicode escape is missing `}`"
    );
    assert_eq!(
        decode_escape("u{110000}").err().unwrap(),
        "Unicode escape is not a scalar value"
    );
}
