use veac_artifact::{ContentDigest, DigestAlgorithm};

use super::*;

#[test]
fn media_producer_binds_tool_configuration_and_workflow_contract() {
    let first = fingerprint("version", b"first");
    let second = fingerprint("version", b"second");
    let first_producer = media_artifact_producer(&first).unwrap();
    let second_producer = media_artifact_producer(&second).unwrap();
    assert_eq!(first_producer.name, "veac-ffmpeg");
    assert_ne!(first_producer.configuration, first.configuration);
    assert_ne!(first_producer.configuration, second_producer.configuration);

    let empty = fingerprint(" ", b"first");
    assert!(media_artifact_producer(&empty).is_err());
    let invalid = FfmpegFingerprint {
        version: "version".into(),
        configuration: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: "invalid".into(),
        },
    };
    assert!(media_artifact_producer(&invalid).is_err());
}

#[test]
fn media_producer_binds_codegen_render_contract() {
    let value = fingerprint("version", b"configuration");
    let current = media_artifact_producer_for_contract(
        &value,
        veac_codegen::RENDER_IMPLEMENTATION_CONTRACT_VERSION,
        &ContentDigest::sha256(b"codegen"),
        &ContentDigest::sha256(b"runtime"),
    )
    .unwrap();
    let previous = media_artifact_producer_for_contract(
        &value,
        veac_codegen::RENDER_IMPLEMENTATION_CONTRACT_VERSION.wrapping_add(1),
        &ContentDigest::sha256(b"codegen"),
        &ContentDigest::sha256(b"runtime"),
    )
    .unwrap();
    assert_ne!(current.configuration, previous.configuration);
}

#[test]
fn media_producer_binds_codegen_and_runtime_build_identities() {
    let value = fingerprint("version", b"configuration");
    let producer = |codegen: &[u8], runtime: &[u8]| {
        media_artifact_producer_for_contract(
            &value,
            veac_codegen::RENDER_IMPLEMENTATION_CONTRACT_VERSION,
            &ContentDigest::sha256(codegen),
            &ContentDigest::sha256(runtime),
        )
        .unwrap()
    };
    let expected = producer(b"codegen", b"runtime");
    assert_ne!(producer(b"changed", b"runtime"), expected);
    assert_ne!(producer(b"codegen", b"changed"), expected);
}

#[test]
fn media_producer_rejects_invalid_backend_identities() {
    let value = fingerprint("version", b"configuration");
    let invalid = ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: "invalid".into(),
    };
    let valid = ContentDigest::sha256(b"valid");
    let producer = |codegen, runtime| {
        media_artifact_producer_for_contract(
            &value,
            veac_codegen::RENDER_IMPLEMENTATION_CONTRACT_VERSION,
            codegen,
            runtime,
        )
    };
    assert!(producer(&invalid, &valid).is_err());
    assert!(producer(&valid, &invalid).is_err());
}

fn fingerprint(version: &str, configuration: &[u8]) -> FfmpegFingerprint {
    FfmpegFingerprint {
        version: version.into(),
        configuration: ContentDigest::sha256(configuration),
    }
}
