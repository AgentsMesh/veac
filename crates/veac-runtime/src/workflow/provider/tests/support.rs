use std::path::{Path, PathBuf};

use serde_json::json;
use veac_artifact::{artifact_key, ArtifactKind, ArtifactRecord, ContentDigest};
use veac_ir::{Rational, RationalTime, TimeRange};
use veac_provider::*;

pub(super) struct Fixture {
    pub manifest: ProviderManifest,
    pub request: ProviderRequestEnvelope,
    pub response: ProviderResponseEnvelope,
    pub payload_name: String,
}

pub(super) fn fixture() -> Fixture {
    let fingerprint = ProviderFingerprint {
        provider: "fake-provider".into(),
        implementation_version: "1.0.0".into(),
        model: "fake-voice".into(),
        model_version: "2026-07".into(),
        configuration: ContentDigest::sha256(b"fake-configuration"),
    };
    let manifest = ProviderManifest::new(
        fingerprint.clone(),
        vec![CapabilityOffer {
            capability: Capability::TextToSpeech,
            contract_versions: vec![CAPABILITY_CONTRACT_VERSION],
            deterministic: true,
        }],
    );
    let request = ProviderRequestEnvelope::new(
        NegotiatedCapability {
            capability: Capability::TextToSpeech,
            contract_version: CAPABILITY_CONTRACT_VERSION,
            provider: fingerprint.clone(),
        },
        ProviderRequest::TextToSpeech(TtsRequest {
            text: "hello".into(),
            voice: VoiceSpec {
                voice: "voice-1".into(),
                language: "en".into(),
                speaking_rate: Rational::new(1, 1).unwrap(),
                pitch_semitones: 0.0,
            },
            sample_rate: 48_000,
            target_range: Some(range()),
        }),
    )
    .unwrap();
    let descriptor = provider_artifact_descriptor(
        ArtifactKind::Speech,
        &fingerprint,
        request_hash(&request).unwrap(),
        Vec::new(),
        json!({"format": "raw-test"}),
    )
    .unwrap();
    let record = ArtifactRecord {
        key: artifact_key(&descriptor).unwrap(),
        content: ContentDigest::sha256(b"speech"),
        size_bytes: 6,
    };
    let response = ProviderResponseEnvelope::new(
        &request,
        ProviderOutput::TextToSpeech(SpeechResult {
            audio: ProviderArtifact {
                role: "speech".into(),
                descriptor,
                record: record.clone(),
            },
            range: range(),
            words: Vec::new(),
        }),
    )
    .unwrap();
    Fixture {
        manifest,
        request,
        response,
        payload_name: format!("{}.payload", record.key.value),
    }
}

#[cfg(unix)]
pub(super) fn provider_script(root: &Path, fixture: &Fixture) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let manifest = shell(
        &String::from_utf8(canonical_provider_manifest_bytes(&fixture.manifest).unwrap()).unwrap(),
    );
    let response =
        shell(&String::from_utf8(canonical_response_bytes(&fixture.response).unwrap()).unwrap());
    let request =
        shell(&String::from_utf8(canonical_request_bytes(&fixture.request).unwrap()).unwrap());
    let name = shell(&fixture.payload_name);
    let script = format!(
        r#"#!/bin/sh
	if [ "$VEAC_PROVIDER_MODE" = "manifest" ]; then
	  [ "$1" = "manifest-nonzero" ] && exit 7
	  [ "$1" = "manifest-oversize" ] && dd if=/dev/zero bs=1048576 count=17 2>/dev/null && exit 0
	  [ "$1" = "manifest-stderr" ] && dd if=/dev/zero bs=1024 count=2 1>&2 2>/dev/null
	  [ "$1" = "manifest-timeout" ] && sleep 2
	  [ "$1" = "shared-timeout" ] && sleep 0.2
	  printf '%s' {manifest}
  [ "$1" = "manifest-polluted" ] && printf '\n'
  exit 0
fi
[ "$1" = "shared-timeout" ] && sleep 0.2
request=$(cat)
[ "$1" = "response-mismatch" ] || [ "$request" = {request} ] || exit 9
payload="$VEAC_PROVIDER_STAGING"/{name}
case "$1" in
  valid) printf '%s' speech > "$payload" ;;
  polluted) printf '%s' speech > "$payload"; printf '%s\n' {response}; exit 0 ;;
  missing) printf '%s' {response}; exit 0 ;;
  extra) printf '%s' speech > "$payload"; printf x > "$VEAC_PROVIDER_STAGING/extra" ;;
  symlink) ln -s /dev/null "$payload" ;;
  directory) mkdir "$payload" ;;
  size) printf '%s' short > "$payload" ;;
  digest) printf '%s' broken > "$payload" ;;
  nonzero) exit 7 ;;
  *) printf '%s' speech > "$payload" ;;
esac
printf '%s' {response}
"#
    );
    let path = root.join("fake-provider.sh");
    std::fs::write(&path, script).unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&path, permissions).unwrap();
    path
}

fn shell(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn range() -> TimeRange {
    TimeRange::new(
        RationalTime::new(0, 100).unwrap(),
        RationalTime::new(50, 100).unwrap(),
    )
    .unwrap()
}
