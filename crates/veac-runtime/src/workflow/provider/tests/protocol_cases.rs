use veac_provider::{canonical_provider_manifest_bytes, canonical_response_bytes, ProviderRequest};

use super::super::protocol::{decode_manifest, decode_response};
use super::super::WorkflowErrorKind;
use super::support::fixture;

#[test]
fn strict_protocol_decoders_accept_only_canonical_bound_envelopes() {
    let fixture = fixture();
    let manifest = canonical_provider_manifest_bytes(&fixture.manifest).unwrap();
    let response = canonical_response_bytes(&fixture.response).unwrap();
    assert_eq!(decode_manifest(&manifest).unwrap(), fixture.manifest);
    assert_eq!(
        decode_response(&response, &fixture.request).unwrap(),
        fixture.response
    );

    for bytes in [b"{".as_slice(), br#"{"schema":1,"schema":2}"#] {
        assert_eq!(
            decode_manifest(bytes).unwrap_err().kind,
            WorkflowErrorKind::ProtocolViolation
        );
    }
    assert_eq!(
        decode_manifest(b"\xff").unwrap_err().kind,
        WorkflowErrorKind::ProtocolViolation
    );

    let mut polluted = manifest.clone();
    polluted.push(b'\n');
    assert_eq!(
        decode_manifest(&polluted).unwrap_err().kind,
        WorkflowErrorKind::ProtocolViolation
    );
    let mut polluted = response.clone();
    polluted.push(b'\n');
    assert_eq!(
        decode_response(&polluted, &fixture.request)
            .unwrap_err()
            .kind,
        WorkflowErrorKind::ProtocolViolation
    );

    let mut invalid = fixture.manifest.clone();
    invalid.schema = "invalid".into();
    let bytes = serde_json_canonicalizer::to_vec(&invalid).unwrap();
    assert_eq!(
        decode_manifest(&bytes).unwrap_err().kind,
        WorkflowErrorKind::ProtocolViolation
    );

    let mut changed = fixture.request.clone();
    let ProviderRequest::TextToSpeech(request) = &mut changed.request else {
        unreachable!()
    };
    request.text = "different".into();
    assert_eq!(
        decode_response(&response, &changed).unwrap_err().kind,
        WorkflowErrorKind::ProtocolViolation
    );
}
