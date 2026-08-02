use std::error::Error;

use veac_artifact::ContentDigest;

use crate::validation;
use crate::*;

#[test]
fn provider_errors_expose_kind_message_and_source() {
    let error = validation::invalid::<()>("bad provider contract").unwrap_err();
    assert_eq!(error.kind, ProviderErrorKind::InvalidContract);
    assert_eq!(error.to_string(), "bad provider contract");
    assert!(error.source().is_none());

    let json = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let error: ProviderError = json.into();
    assert_eq!(error.kind, ProviderErrorKind::Serialization);
    assert!(error.source().is_some());

    let mut digest = ContentDigest::sha256(b"bad");
    digest.value.clear();
    let artifact = digest.validate().unwrap_err();
    let error: ProviderError = artifact.into();
    assert_eq!(error.kind, ProviderErrorKind::Artifact);
    assert!(error.source().is_some());
}

#[test]
fn primitive_validation_rejects_noncanonical_values() {
    assert!(validation::text(" ", "value").is_err());
    assert!(validation::language("en--US").is_err());
    assert!(validation::language(&"a".repeat(64)).is_err());
    assert!(validation::finite(f64::NAN, "value").is_err());
    assert!(validation::probability(1.1, "value").is_err());
    assert!(validation::positive(0.0, "value").is_err());
    assert!(validation::time(veac_ir::RationalTime {
        value: 0,
        timescale: 0
    })
    .is_err());
    assert!(validation::range(veac_ir::TimeRange {
        start: veac_ir::RationalTime {
            value: 0,
            timescale: 1
        },
        duration: veac_ir::RationalTime {
            value: 0,
            timescale: 1
        },
    })
    .is_err());
}
