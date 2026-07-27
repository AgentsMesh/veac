use std::error::Error;

use super::*;

#[test]
fn invalid_cached_descriptor_errors_retain_the_validation_source() {
    let source = ArtifactError::new(ArtifactErrorKind::InvalidContract, "invalid descriptor");
    let error = cache_metadata(source);
    assert_eq!(error.kind, ArtifactErrorKind::CorruptCache);
    assert!(error.source().is_some());
}
