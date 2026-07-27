use std::error::Error;

use super::super::{staging, ProviderResourceLimits, WorkflowErrorKind};
use super::support::fixture;

#[test]
fn staging_guard_cancels_during_payload_hashing() {
    let temp = tempfile::tempdir().unwrap();
    let mut fixture = fixture();
    let payload = vec![0x5a; 256 * 1024];
    std::fs::write(temp.path().join(&fixture.payload_name), &payload).unwrap();
    let veac_provider::ProviderOutput::TextToSpeech(result) = &mut fixture.response.output else {
        unreachable!()
    };
    result.audio.record.content = veac_artifact::ContentDigest::sha256(&payload);
    result.audio.record.size_bytes = payload.len() as u64;
    let mut calls = 0;
    let error = staging::verify_while(
        temp.path(),
        &fixture.response,
        ProviderResourceLimits::default(),
        || {
            calls += 1;
            calls < 18
        },
    )
    .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
    assert_eq!(calls, 18);
    assert!(error.source().unwrap().to_string().contains("deadline"));
}
