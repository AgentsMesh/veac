use veac_artifact::*;
use veac_ir::*;
use veac_provider::{
    ApplicationHeader, AsrCaptionApplication, CaptionProposalContext, ProposalEvidence,
    ProviderEditProposal,
};
use veac_runtime::executor::{FfmpegEnvironment, SystemFfmpeg};

use super::support::*;
use super::workflow_provider_support::*;

#[cfg(unix)]
#[test]
fn derive_command_generates_and_reuses_a_content_addressed_artifact() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("input.bin");
    let request_file = temp.path().join("request.json");
    let store_root = temp.path().join("store");
    let ffmpeg = fake_ffmpeg(temp.path());
    let ffprobe = fake_audio_ffprobe(temp.path(), 1);
    let fingerprint = FfmpegEnvironment::fingerprint(&SystemFfmpeg::new(&ffmpeg)).unwrap();
    std::fs::write(&input, b"source").unwrap();
    let request = MediaArtifactRequest {
        source_identity: ContentDigest::sha256(b"source"),
        producer: veac_runtime::workflow::media_artifact_producer(&fingerprint).unwrap(),
        spec: MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: veac_ir::StreamSelection {
                global_index: 1,
                type_index: 0,
            },
            source_clock: veac_artifact::SourceClockSpec::Identity {
                duration: RationalTime::new(1_000, 1_000).unwrap(),
            },
            sample_rate: 48_000,
            channels: 2,
        }),
    };
    std::fs::write(&request_file, serde_json::to_vec(&request).unwrap()).unwrap();
    let run = || {
        veac()
            .args([
                "derive",
                input.to_str().unwrap(),
                request_file.to_str().unwrap(),
                "--store",
                store_root.to_str().unwrap(),
                "--ffmpeg",
                ffmpeg.to_str().unwrap(),
                "--ffprobe",
                ffprobe.to_str().unwrap(),
            ])
            .output()
            .unwrap()
    };
    let first = run();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first: veac_runtime::workflow::GeneratedArtifact =
        serde_json::from_slice(&first.stdout).unwrap();
    assert!(!first.cache_hit);
    assert_eq!(
        ArtifactStore::new(&store_root)
            .get(&first.record.key)
            .unwrap()
            .unwrap()
            .payload,
        b"derived"
    );
    let second: veac_runtime::workflow::GeneratedArtifact =
        serde_json::from_slice(&run().stdout).unwrap();
    assert!(second.cache_hit);
    assert_eq!(second.record, first.record);
}

#[cfg(unix)]
#[test]
fn provider_commands_execute_protocol_and_emit_a_caption_proposal() {
    let temp = tempdir().unwrap();
    let fixture = provider_fixture();
    let request = temp.path().join("provider-request.json");
    let response = temp.path().join("provider-response.json");
    let executed = temp.path().join("executed-response.json");
    std::fs::write(
        &request,
        veac_provider::canonical_request_bytes(&fixture.request).unwrap(),
    )
    .unwrap();
    std::fs::write(
        &response,
        veac_provider::canonical_response_bytes(&fixture.response).unwrap(),
    )
    .unwrap();
    let provider = fake_provider(temp.path(), &fixture);
    veac()
        .args([
            "provider-run",
            request.to_str().unwrap(),
            "--program",
            provider.to_str().unwrap(),
            "--store",
            temp.path().join("store").to_str().unwrap(),
            "--response",
            executed.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(
        std::fs::read(&executed).unwrap(),
        with_newline(std::fs::read(&response).unwrap())
    );

    let project_file = caption_project(&temp);
    let project =
        veac_ir::decode_canonical_json(&std::fs::read_to_string(&project_file).unwrap()).unwrap();
    let visual = project.project.sequences[0].tracks[0].clips[0]
        .visual
        .clone()
        .unwrap();
    let context = CaptionProposalContext::AsrCaptions(Box::new(AsrCaptionApplication {
        header: ApplicationHeader {
            project_revision: project.project.revision,
            operation_id: OperationId::new("op_provider_cli").unwrap(),
        },
        sequence_id: project.project.sequences[0].id.clone(),
        track_id: TrackId::new("trk_provider_captions").unwrap(),
        style: TextStyle::default(),
        visual,
        item_id_prefix: "itm_provider_".into(),
    }));
    let context_file = temp.path().join("context.json");
    let proposal_file = temp.path().join("proposal.json");
    std::fs::write(&context_file, serde_json::to_vec(&context).unwrap()).unwrap();
    veac()
        .args([
            "provider-propose",
            project_file.to_str().unwrap(),
            request.to_str().unwrap(),
            response.to_str().unwrap(),
            context_file.to_str().unwrap(),
            "--output",
            proposal_file.to_str().unwrap(),
        ])
        .assert()
        .success();
    let proposal: ProviderEditProposal =
        serde_json::from_slice(&std::fs::read(proposal_file).unwrap()).unwrap();
    assert_eq!(proposal.batch.operations.len(), 1);
    assert!(matches!(
        &proposal.evidence[0],
        ProposalEvidence::AsrSegment { segment_id, .. } if segment_id == "segment-1"
    ));
}

fn caption_project(temp: &TempDir) -> std::path::PathBuf {
    let path = compile_ir(temp, GENERATED_SOURCE);
    let mut project =
        veac_ir::decode_canonical_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
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
    project.project.sequences[0].authorship = None;
    std::fs::write(&path, veac_ir::canonical_json(&project).unwrap()).unwrap();
    path
}

fn with_newline(mut value: Vec<u8>) -> Vec<u8> {
    value.push(b'\n');
    value
}
