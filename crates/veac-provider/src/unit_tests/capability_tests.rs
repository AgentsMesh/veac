use veac_artifact::ContentDigest;

use crate::test_support::fingerprint;
use crate::*;

fn offer(capability: Capability) -> CapabilityOffer {
    CapabilityOffer {
        capability,
        contract_versions: vec![1],
        deterministic: true,
    }
}

fn manifest() -> ProviderManifest {
    ProviderManifest::new(
        fingerprint(),
        vec![offer(Capability::Asr), offer(Capability::Translation)],
    )
}

#[test]
fn negotiates_highest_shared_deterministic_version() {
    let mut value = manifest();
    value.offers[0].contract_versions = vec![1, 2, 3];
    let selected = negotiate(
        &value,
        &CapabilityRequirement {
            capability: Capability::Asr,
            accepted_versions: vec![1, 3],
            deterministic: true,
        },
    )
    .unwrap();
    assert_eq!(selected.contract_version, 3);
    assert_eq!(selected.provider, fingerprint());

    let current = CapabilityRequirement::current(Capability::Translation);
    assert_eq!(negotiate(&value, &current).unwrap().contract_version, 1);
    let bytes = canonical_provider_manifest_bytes(&value).unwrap();
    assert_eq!(
        serde_json::from_slice::<ProviderManifest>(&bytes).unwrap(),
        value
    );
    assert_eq!(
        provider_manifest_hash(&value).unwrap(),
        ContentDigest::sha256(bytes)
    );
}

#[test]
fn negotiation_reports_each_unsatisfied_requirement() {
    let value = manifest();
    let mut requirement = CapabilityRequirement::current(Capability::Dubbing);
    assert_eq!(
        negotiate(&value, &requirement).unwrap_err().kind,
        ProviderErrorKind::UnsupportedCapability
    );

    requirement.capability = Capability::Asr;
    requirement.accepted_versions = vec![9];
    assert_eq!(
        negotiate(&value, &requirement).unwrap_err().kind,
        ProviderErrorKind::VersionMismatch
    );

    let mut nondeterministic = value.clone();
    nondeterministic.offers[0].deterministic = false;
    requirement.accepted_versions = vec![1];
    assert_eq!(
        negotiate(&nondeterministic, &requirement).unwrap_err().kind,
        ProviderErrorKind::NondeterministicProvider
    );

    requirement.accepted_versions.clear();
    assert_eq!(
        negotiate(&value, &requirement).unwrap_err().kind,
        ProviderErrorKind::InvalidContract
    );
    requirement.accepted_versions = vec![0];
    assert!(negotiate(&value, &requirement).is_err());
}

#[test]
fn manifest_rejects_unpinned_or_ambiguous_offers() {
    let mut value = manifest();
    value.schema = "future".into();
    assert!(value.validate().is_err());

    value = manifest();
    value.offers.swap(0, 1);
    assert!(value.validate().is_err());
    value = manifest();
    value.offers.push(offer(Capability::Translation));
    assert!(value.validate().is_err());

    value = manifest();
    value.offers[0].contract_versions.clear();
    assert!(value.validate().is_err());
    value.offers[0].contract_versions = vec![0];
    assert!(value.validate().is_err());
    value.offers[0].contract_versions = vec![2, 1];
    assert!(value.validate().is_err());
}

#[test]
fn fingerprint_requires_every_pin_and_valid_configuration() {
    let mut value = fingerprint();
    assert_eq!(
        value.artifact_producer().name,
        "fixture-provider/fixture-model"
    );
    value.provider.clear();
    assert!(value.validate().is_err());
    value = fingerprint();
    value.implementation_version.clear();
    assert!(value.validate().is_err());
    value = fingerprint();
    value.model.clear();
    assert!(value.validate().is_err());
    value = fingerprint();
    value.model_version.clear();
    assert!(value.validate().is_err());
    value = fingerprint();
    value.configuration = ContentDigest::sha256(b"x");
    value.configuration.value.pop();
    assert!(value.validate().is_err());
}
