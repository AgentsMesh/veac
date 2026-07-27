use crate::ResolutionErrorKind;

use super::super::error::nested_component_missing;

#[test]
fn nested_component_diagnostic_preserves_context() {
    let diagnostic = nested_component_missing(
        "/project/sequences/seq_nested/tracks/*/clips/itm_nested/source",
        "seq_nested",
        "video",
    );

    assert_eq!(
        diagnostic.kind,
        ResolutionErrorKind::NestedSequenceComponentUnavailable
    );
    assert_eq!(diagnostic.code, "NESTED_SEQUENCE_VIDEO_UNAVAILABLE");
    assert_eq!(diagnostic.object_id.as_deref(), Some("seq_nested"));
    assert_eq!(
        diagnostic.pointer,
        "/project/sequences/seq_nested/tracks/*/clips/itm_nested/source"
    );
    assert_eq!(
        diagnostic.message,
        "nested sequence seq_nested has no rendered video component"
    );
}
