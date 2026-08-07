use super::*;

#[test]
fn source_spans_reject_each_u32_overflow_boundary() {
    let maximum = u32::MAX as usize;
    let value = span(0..maximum).unwrap();
    assert_eq!(value.start, 0);
    assert_eq!(value.end, u32::MAX);

    let overflow = maximum
        .checked_add(1)
        .expect("coverage target uses 64-bit usize");
    for range in [overflow..overflow, 0..overflow] {
        let error = span(range).unwrap_err();
        assert_eq!(error.reason_code(), "EXECUTABLE_TEMPORAL_PROVENANCE");
        assert!(error.to_string().contains("source span exceeds u32"));
    }
}
