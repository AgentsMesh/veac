use veac_artifact::*;
use veac_ir::{Rational, RationalTime, StreamSelection};
use veac_runtime::workflow::{MediaWorkflow, WorkflowErrorKind};

use super::support::*;

#[test]
fn analysis_source_identity_parameter_changes_and_corruption_fail_closed() {
    let temp = tempfile::tempdir().unwrap();
    let input = media_fixture(temp.path());
    let store_root = temp.path().join("store");
    let store = ArtifactStore::new(&store_root);
    let workflow = MediaWorkflow::new("ffmpeg");
    let analysis = analysis(&input);
    let first = workflow.ingest_analysis(&store, &input, &analysis).unwrap();
    let payload = store.get(&first.record.key).unwrap().unwrap().payload;
    let decoded: AnalysisResultEnvelope = serde_json::from_slice(&payload).unwrap();
    assert_eq!(decoded, analysis.result);
    assert!(
        workflow
            .ingest_analysis(&store, &input, &analysis)
            .unwrap()
            .cache_hit
    );

    let mut wrong_source = analysis.clone();
    wrong_source.source_identity = ContentDigest::sha256(b"wrong");
    assert_eq!(
        workflow
            .ingest_analysis(&store, &input, &wrong_source)
            .unwrap_err()
            .kind,
        WorkflowErrorKind::SourceIdentityMismatch
    );

    std::fs::write(
        cache_directory(&store_root, &first.record.key).join("payload.bin"),
        b"corrupt",
    )
    .unwrap();
    assert_eq!(
        workflow
            .ingest_analysis(&store, &input, &analysis)
            .unwrap_err()
            .kind,
        WorkflowErrorKind::Artifact
    );
}

fn analysis(input: &std::path::Path) -> AnalysisIngestionRequest {
    AnalysisIngestionRequest {
        source_identity: ContentDigest::sha256(std::fs::read(input).unwrap()),
        producer: ProducerFingerprint {
            name: "scene-provider".into(),
            version: "1".into(),
            configuration: ContentDigest::sha256(b"scene-provider-configuration"),
        },
        result: AnalysisResultEnvelope::new(
            AnalysisDescriptor::SceneBoundaries(SceneBoundaryAnalysisDescriptor {
                sensitivity_millionths: 500_000,
            }),
            AnalysisResult::SceneBoundaries(SceneBoundaryAnalysisResult {
                boundaries: vec![SceneBoundary {
                    at: RationalTime::new(4, 10).unwrap(),
                    confidence_millionths: 900_000,
                }],
            }),
        )
        .unwrap(),
    }
}

#[test]
fn real_media_workflow_rejects_hls_child_resources() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("playlist.m3u8");
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let child = format!("http://{}/child.ts", listener.local_addr().unwrap());
    std::fs::write(
        &input,
        format!("#EXTM3U\n#EXT-X-TARGETDURATION:1\n#EXTINF:1,\n{child}\n#EXT-X-ENDLIST\n"),
    )
    .unwrap();
    let request = request(
        &input,
        MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream: stream(0, 0),
            source_clock: SourceClockSpec::Identity {
                duration: time(100),
            },
            width: 16,
            height: 16,
            frame_rate: Rational::new(1, 1).unwrap(),
            crf: 28,
        }),
    );

    let store_root = temp.path().join("store");
    let error = MediaWorkflow::new("ffmpeg")
        .derive(&ArtifactStore::new(&store_root), &input, &request)
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ToolFailure);
    assert_eq!(error.to_string(), "media source preflight failed");
    assert!(!contains_payload(&store_root));
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 100).unwrap()
}

fn stream(global_index: u32, type_index: u32) -> StreamSelection {
    StreamSelection {
        global_index,
        type_index,
    }
}

fn contains_payload(path: &std::path::Path) -> bool {
    std::fs::read_dir(path).is_ok_and(|entries| {
        entries.filter_map(Result::ok).any(|entry| {
            entry.file_name() == "payload.bin"
                || (entry.path().is_dir() && contains_payload(&entry.path()))
        })
    })
}
