use super::encode;

#[test]
fn ass_font_encoding_has_the_expected_partial_groups() {
    assert_eq!(encode(&[0]), "!!");
    assert_eq!(encode(&[0, 0]), "!!!");
    assert_eq!(encode(&[0, 0, 0]), "!!!!");
    assert_eq!(encode(&[255, 255, 255]), "````");
}
