use assert_cmd::Command;
use veac_artifact::*;

#[test]
fn cli_ingests_typed_analysis_source_to_verified_artifact() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("source.mov");
    let request_path = temp.path().join("analysis.json");
    let store = temp.path().join("store");
    std::fs::write(&input, b"source-bytes").unwrap();
    let request = request(ContentDigest::sha256(b"source-bytes"));
    std::fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();

    let output = veac()
        .args([
            "ingest-analysis",
            input.to_str().unwrap(),
            request_path.to_str().unwrap(),
            "--store",
            store.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated: veac_runtime::workflow::GeneratedArtifact =
        serde_json::from_slice(&output.stdout).unwrap();
    assert!(!generated.cache_hit);
    let stored = ArtifactStore::new(&store)
        .get(&generated.record.key)
        .unwrap()
        .unwrap();
    let decoded: AnalysisResultEnvelope = serde_json::from_slice(&stored.payload).unwrap();
    assert_eq!(decoded, request.result);

    let second = veac()
        .args([
            "ingest-analysis",
            input.to_str().unwrap(),
            request_path.to_str().unwrap(),
            "--store",
            store.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let cached: veac_runtime::workflow::GeneratedArtifact =
        serde_json::from_slice(&second.stdout).unwrap();
    assert!(cached.cache_hit);
    assert_eq!(cached.record, generated.record);
}

fn request(source_identity: ContentDigest) -> AnalysisIngestionRequest {
    AnalysisIngestionRequest {
        source_identity,
        producer: ProducerFingerprint {
            name: "integration-analyzer".into(),
            version: "2026.08".into(),
            configuration: ContentDigest::sha256(b"analyzer-configuration"),
        },
        result: AnalysisResultEnvelope::new(
            AnalysisDescriptor::BeatMarkers(BeatMarkerAnalysisDescriptor {
                minimum_bpm_milli: 60_000,
                maximum_bpm_milli: 180_000,
            }),
            AnalysisResult::BeatMarkers(BeatMarkerAnalysisResult {
                markers: vec![BeatMarker {
                    at: veac_ir::RationalTime::new(1, 2).unwrap(),
                    confidence_millionths: 875_000,
                }],
            }),
        )
        .unwrap(),
    }
}

fn veac() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("veac"))
}
