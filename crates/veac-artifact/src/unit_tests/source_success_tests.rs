use crate::*;

#[test]
fn bounded_source_entry_points_complete_verified_success_paths() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let first_copy = temp.path().join("first-copy");
    let second_copy = temp.path().join("second-copy");
    let payload = vec![0x5a; 70 * 1024];
    std::fs::write(&source, &payload).unwrap();
    let expected = test_support::media_identity(&payload);
    let limit = payload.len() as u64;

    assert_eq!(
        read_verified_source_bounded_while(&source, Some(&expected), limit, || true).unwrap(),
        payload
    );
    assert_eq!(
        verify_source_bounded(&source, Some(&expected), limit)
            .unwrap()
            .identity,
        expected
    );
    assert_eq!(
        verify_source_bounded_while(&source, Some(&expected), limit, || true)
            .unwrap()
            .size_bytes,
        limit
    );
    copy_verified_source_bounded(&source, &first_copy, Some(&expected), limit).unwrap();
    copy_verified_source_bounded_while(&source, &second_copy, Some(&expected), limit, || true)
        .unwrap();
    assert_eq!(std::fs::read(first_copy).unwrap(), payload);
    assert_eq!(std::fs::read(second_copy).unwrap(), payload);
}
