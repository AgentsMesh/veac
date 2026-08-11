use super::*;

#[test]
fn reads_exact_document_types_from_bounded_headers() {
    assert_eq!(doc_type(&header(b"webm")), Some(b"webm".as_slice()));
    assert_eq!(doc_type(&header(b"matroska")), Some(b"matroska".as_slice()));

    assert!(validate(&header(b"webm"), DocType::Webm).is_ok());
    assert!(validate(&header(b"matroska"), DocType::Matroska).is_ok());
    assert!(validate(&header(b"webm"), DocType::Matroska).is_err());
}

#[test]
fn rejects_malformed_unknown_and_truncated_elements() {
    assert!(doc_type(b"not-ebml").is_none());
    assert!(doc_type(&EBML_MAGIC).is_none());
    let mut value = header(b"webm");
    value[4] = 0xff;
    assert!(doc_type(&value).is_none());
    assert!(doc_type(&header(b"other")).is_some());

    let truncated_header = [EBML_MAGIC.as_slice(), &[0x88, 0x42, 0x82, 0x80]].concat();
    assert!(doc_type(&truncated_header).is_none());
    let oversized_element = [EBML_MAGIC.as_slice(), &[0x83, 0x42, 0x82, 0x84]].concat();
    assert!(doc_type(&oversized_element).is_none());
    let unrelated_element = [EBML_MAGIC.as_slice(), &[0x83, 0x42, 0x83, 0x80]].concat();
    assert!(doc_type(&unrelated_element).is_none());
}

#[test]
fn vint_rejects_invalid_shapes_and_preserves_element_markers() {
    assert_eq!(vint(&[0x84], false), Some((4, 1)));
    assert_eq!(vint(&[0x42, 0x82], true), Some((0x4282, 2)));
    assert!(vint(&[0], false).is_none());
    assert!(vint(&[0x40], false).is_none());
    assert!(vint(&[0xff], false).is_none());
}

fn header(value: &[u8]) -> Vec<u8> {
    let content_len = 3 + value.len();
    let mut bytes = EBML_MAGIC.to_vec();
    bytes.push(0x80 | u8::try_from(content_len).unwrap());
    bytes.extend([0x42, 0x82, 0x80 | u8::try_from(value.len()).unwrap()]);
    bytes.extend(value);
    bytes
}
