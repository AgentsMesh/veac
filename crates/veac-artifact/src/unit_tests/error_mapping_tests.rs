use std::error::Error;

use crate::{ArtifactError, ArtifactErrorKind};
use serde::Serialize;

#[test]
fn canonical_contract_error_adapters_preserve_sources_and_classification() {
    for error in [
        crate::binding::serialization(malformed_json()),
        crate::json::canonical_bounded(&Fails, 10, "expected failure").unwrap_err(),
        crate::manifest::serialization_error(malformed_json()),
        crate::render_segment::serialization_error(malformed_json()),
    ] {
        assert_serialization(error);
    }
    assert_eq!(
        crate::materialize::size_overflow().kind,
        ArtifactErrorKind::InvalidContract
    );
}

struct Fails;

impl Serialize for Fails {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom("expected failure"))
    }
}

fn malformed_json() -> serde_json::Error {
    serde_json::from_str::<serde_json::Value>("{").unwrap_err()
}

fn assert_serialization(error: ArtifactError) {
    assert_eq!(error.kind, ArtifactErrorKind::Serialization);
    assert!(error.source().is_some());
}
