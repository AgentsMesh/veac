use veac_evidence::*;
use veac_ir::{HashAlgorithm, MediaIdentity};
use veac_runtime::observation::{MediaObserver, ObservationSource};

pub fn binding() -> BoundObservationSource {
    BoundObservationSource {
        source_id: "final".into(),
        source: ObservationSource {
            path: "fixture.mov".into(),
            identity: MediaIdentity {
                algorithm: HashAlgorithm::Sha256,
                digest: "0".repeat(64),
            },
            video_stream: None,
        },
    }
}

pub fn plan_and_provenance(
    suite: &EvidenceSuiteV1,
) -> (ValidatedSuite, ObservationPlanV1, EvidenceProvenanceV1) {
    let validated = validate(suite.clone()).unwrap();
    let plan = plan_observations(&validated).unwrap();
    let producer = EvidenceProducerFingerprint {
        engine: "ffmpeg".into(),
        version: "fixture".into(),
        configuration_sha256: "1".repeat(64),
    };
    let provenance = build_provenance(&plan, &[binding()], producer, None).unwrap();
    (validated, plan, provenance)
}
