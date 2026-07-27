use std::error::Error;

use super::time_error;
use crate::ArtifactErrorKind;

#[test]
fn time_failures_keep_the_ir_error_as_their_source() {
    let error = time_error(veac_ir::TimeError::Overflow);
    assert_eq!(error.kind, ArtifactErrorKind::InvalidContract);
    assert!(error.source().is_some());
}
