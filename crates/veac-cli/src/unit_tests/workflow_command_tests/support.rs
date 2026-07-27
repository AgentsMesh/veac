use std::path::{Path, PathBuf};

use veac_artifact::{ContentDigest, MediaArtifactRequest, MediaArtifactSpec};
use veac_ir::*;
use veac_provider::*;
use veac_runtime::executor::{FfmpegEnvironment, SystemFfmpeg};

#[path = "support/media_tools.rs"]
mod media_tools;
pub(super) use media_tools::*;

pub(super) struct ProviderFixture {
    pub request: ProviderRequestEnvelope,
    pub response: ProviderResponseEnvelope,
    manifest: ProviderManifest,
}

pub(super) fn artifact_request_bytes(ffmpeg: &Path) -> Vec<u8> {
    let fingerprint = FfmpegEnvironment::fingerprint(&SystemFfmpeg::new(ffmpeg)).unwrap();
    serde_json::to_vec(&MediaArtifactRequest {
        source_identity: ContentDigest::sha256(b"source"),
        producer: veac_runtime::workflow::media_artifact_producer(&fingerprint).unwrap(),
        spec: MediaArtifactSpec::ProxyAudio(veac_artifact::ProxyAudioSpec {
            source_stream: veac_ir::StreamSelection {
                global_index: 0,
                type_index: 0,
            },
            source_clock: veac_artifact::SourceClockSpec::Identity {
                duration: RationalTime::new(1_000, 1_000).unwrap(),
            },
            sample_rate: 48_000,
            channels: 2,
        }),
    })
    .unwrap()
}

pub(super) fn provider_fixture() -> ProviderFixture {
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
    ProviderFixture {
        request,
        response,
        manifest,
    }
}

#[cfg(unix)]
pub(super) fn fake_provider(root: &Path, fixture: &ProviderFixture) -> PathBuf {
    let manifest = shell(&canonical_provider_manifest_bytes(&fixture.manifest).unwrap());
    let request = shell(&canonical_request_bytes(&fixture.request).unwrap());
    let response = shell(&canonical_response_bytes(&fixture.response).unwrap());
    executable(
        root,
        "provider.sh",
        &format!("#!/bin/sh\nif [ \"$VEAC_PROVIDER_MODE\" = manifest ]; then printf '%s' {manifest}; exit 0; fi\nvalue=$(cat)\n[ \"$value\" = {request} ] || exit 9\nprintf '%s' {response}\n"),
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

pub(super) fn caption_project(temp: &tempfile::TempDir) -> (PathBuf, CaptionProposalContext) {
    let project_file =
        super::super::support::canonical_project(temp, super::super::support::GENERATED_SOURCE);
    let mut project = crate::canonical::load(&project_file).unwrap();
    let visual = project.project.sequences[0].tracks[0].clips[0]
        .visual
        .clone()
        .unwrap();
    project.project.sequences[0].tracks.push(Track {
        id: TrackId::new("trk_provider_captions").unwrap(),
        kind: TrackKind::Caption,
        order: 10,
        placement_mode: PlacementMode::Free,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips: Vec::new(),
    });
    std::fs::write(&project_file, veac_ir::canonical_json(&project).unwrap()).unwrap();
    let context = CaptionProposalContext::AsrCaptions(Box::new(AsrCaptionApplication {
        header: ApplicationHeader {
            project_revision: project.project.revision,
            operation_id: OperationId::new("op_provider_cli").unwrap(),
        },
        sequence_id: SequenceId::new("seq_main").unwrap(),
        track_id: TrackId::new("trk_provider_captions").unwrap(),
        style: TextStyle::default(),
        visual,
        item_id_prefix: "itm_provider_".into(),
    }));
    (project_file, context)
}

fn shell(value: &[u8]) -> String {
    let value = String::from_utf8(value.to_vec()).unwrap();
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn range(start: i64, duration: i64) -> TimeRange {
    TimeRange::new(
        RationalTime::new(start, 100).unwrap(),
        RationalTime::new(duration, 100).unwrap(),
    )
    .unwrap()
}
