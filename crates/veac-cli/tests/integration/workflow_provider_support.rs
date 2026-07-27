use std::path::{Path, PathBuf};

use veac_artifact::ContentDigest;
use veac_ir::{RationalTime, TimeRange};
use veac_provider::*;

pub(super) struct Fixture {
    pub manifest: ProviderManifest,
    pub request: ProviderRequestEnvelope,
    pub response: ProviderResponseEnvelope,
}

pub(super) fn provider_fixture() -> Fixture {
    let fingerprint = ProviderFingerprint {
        provider: "cli-fake-provider".into(),
        implementation_version: "1".into(),
        model: "transcriber".into(),
        model_version: "1".into(),
        configuration: ContentDigest::sha256(b"configuration"),
    };
    let manifest = ProviderManifest::new(
        fingerprint.clone(),
        vec![CapabilityOffer {
            capability: Capability::Asr,
            contract_versions: vec![CAPABILITY_CONTRACT_VERSION],
            deterministic: true,
        }],
    );
    let request = ProviderRequestEnvelope::new(
        NegotiatedCapability {
            capability: Capability::Asr,
            contract_version: CAPABILITY_CONTRACT_VERSION,
            provider: fingerprint,
        },
        ProviderRequest::Asr(AsrRequest {
            audio: InputArtifact {
                content: ContentDigest::sha256(b"audio"),
                media_type: MediaType::Audio,
                stream_index: Some(0),
                range: Some(range(0, 100)),
            },
            language_hint: Some("en".into()),
            speakers: SpeakerMode::None,
            word_timing: false,
        }),
    )
    .unwrap();
    let response = ProviderResponseEnvelope::new(
        &request,
        ProviderOutput::Asr(AsrResult {
            language: "en".into(),
            segments: vec![TranscriptSegment {
                id: "segment-1".into(),
                range: range(0, 50),
                speaker: None,
                text: "hello from provider".into(),
                confidence: 0.95,
                words: Vec::new(),
            }],
        }),
    )
    .unwrap();
    Fixture {
        manifest,
        request,
        response,
    }
}

#[cfg(unix)]
pub(super) fn fake_provider(root: &Path, fixture: &Fixture) -> PathBuf {
    let manifest = shell(
        &String::from_utf8(canonical_provider_manifest_bytes(&fixture.manifest).unwrap()).unwrap(),
    );
    let request =
        shell(&String::from_utf8(canonical_request_bytes(&fixture.request).unwrap()).unwrap());
    let response =
        shell(&String::from_utf8(canonical_response_bytes(&fixture.response).unwrap()).unwrap());
    let script = format!(
        r#"#!/bin/sh
if [ "$VEAC_PROVIDER_MODE" = "manifest" ]; then
  printf '%s' {manifest}
  exit 0
fi
value=$(cat)
[ "$value" = {request} ] || exit 9
printf '%s' {response}
"#
    );
    executable(root, "provider.sh", &script)
}

#[cfg(unix)]
pub(super) fn fake_ffmpeg(root: &Path) -> PathBuf {
    executable(
        root,
        "ffmpeg.sh",
        "#!/bin/sh\nif [ \"$1\" = -version ]; then printf 'ffmpeg version cli-integration-v1\\n'; exit 0; fi\nfor output do :; done\nprintf '%s' derived > \"$output\"\n",
    )
}

#[cfg(unix)]
pub(super) fn executable(root: &Path, name: &str, content: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = root.join(name);
    std::fs::write(&path, content).unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&path, permissions).unwrap();
    path
}

fn shell(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn range(start: i64, duration: i64) -> TimeRange {
    TimeRange::new(
        RationalTime::new(start, 100).unwrap(),
        RationalTime::new(duration, 100).unwrap(),
    )
    .unwrap()
}
