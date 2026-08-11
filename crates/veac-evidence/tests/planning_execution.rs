mod support;

use std::collections::BTreeMap;
use std::sync::Mutex;

use veac_evidence::*;
use veac_ir::{HashAlgorithm, MediaIdentity};
use veac_runtime::observation as runtime;

#[derive(Default)]
struct FakeBackend {
    frames: Mutex<usize>,
    decodes: Mutex<usize>,
    fail: bool,
}

impl ObservationBackend for FakeBackend {
    fn frame(&self, request: &runtime::FrameRequest) -> Result<runtime::DecodedFrame, String> {
        *self.frames.lock().unwrap() += 1;
        if self.fail {
            return Err("frame failed".into());
        }
        Ok(runtime::DecodedFrame {
            requested_time: request.time,
            actual_pts: request.time,
            width: 4,
            height: 4,
            pixel_format: runtime::FramePixelFormat::Rgba8,
            bytes: vec![0; 64],
        })
    }

    fn decode(
        &self,
        request: &runtime::DecodeRequest,
    ) -> Result<runtime::DecodeObservation, String> {
        *self.decodes.lock().unwrap() += 1;
        if self.fail {
            return Err("decode failed".into());
        }
        Ok(runtime::DecodeObservation {
            complete: true,
            identity: request.source.identity.clone(),
            decoded_frames: 8,
            last_pts: Some(RationalTime::new(7, 30).unwrap()),
            errors: vec![],
        })
    }
}

#[test]
fn planner_deduplicates_equivalent_frames_and_decode_sources() {
    let mut suite = support::suite();
    suite.samples[0].at = RationalTime::new(2, 60).unwrap();
    suite.samples[1].at = RationalTime::new(1, 30).unwrap();
    let plan = plan_observations(&validate(suite).unwrap()).unwrap();
    assert_eq!(plan.schema_version, 1);
    assert_eq!(plan.suite_sha256.len(), 64);
    assert_eq!(plan.frames.len(), 7);
    assert_eq!(plan.decodes.len(), 1);
    let merged = plan
        .frames
        .iter()
        .find(|frame| frame.sample_ids.len() == 2)
        .unwrap();
    assert_eq!(merged.at, RationalTime::new(1, 30).unwrap());
    assert_eq!(merged.sample_ids, ["overlay", "under"]);
}

#[test]
fn executor_fans_a_frame_out_and_preserves_failure_evidence() {
    let mut suite = support::suite();
    suite.samples[0].at = RationalTime::new(2, 60).unwrap();
    suite.samples[1].at = RationalTime::new(1, 30).unwrap();
    let plan = plan_observations(&validate(suite).unwrap()).unwrap();
    let source_binding = binding();
    let backend = FakeBackend::default();
    let run = execute_observations(
        &plan,
        std::slice::from_ref(&source_binding),
        &backend,
        BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(*backend.frames.lock().unwrap(), 7);
    assert_eq!(*backend.decodes.lock().unwrap(), 1);
    assert_eq!(run.observations.frames.len(), 8);
    assert_eq!(run.observations.decodes["final"].decoded_frames, 8);
    assert!(run.failures.is_empty());

    let failed = FakeBackend {
        fail: true,
        ..FakeBackend::default()
    };
    let run = execute_observations(&plan, &[source_binding], &failed, BTreeMap::new()).unwrap();
    assert_eq!(run.failures.len(), 8);
    assert!(!run.observations.decodes["final"].complete);
    assert_eq!(run.observations.decodes["final"].errors, ["decode failed"]);
}

#[test]
fn executor_rejects_incomplete_or_ambiguous_binding_sets() {
    let plan = plan_observations(&validate(support::suite()).unwrap()).unwrap();
    let backend = FakeBackend::default();
    let missing = execute_observations(&plan, &[], &backend, BTreeMap::new()).unwrap_err();
    assert!(matches!(missing, ObservationRunError::MissingBinding(_)));
    let source_binding = binding();
    let duplicate = execute_observations(
        &plan,
        &[source_binding.clone(), source_binding],
        &backend,
        BTreeMap::new(),
    )
    .unwrap_err();
    assert!(matches!(
        duplicate,
        ObservationRunError::DuplicateBinding(_)
    ));
    let mut unexpected = binding();
    unexpected.source_id = "other".into();
    let error = execute_observations(&plan, &[unexpected], &backend, BTreeMap::new()).unwrap_err();
    assert!(matches!(error, ObservationRunError::UnexpectedBinding(_)));
    assert!(!error.to_string().is_empty());
}

fn binding() -> BoundObservationSource {
    BoundObservationSource {
        source_id: "final".into(),
        source: runtime::ObservationSource {
            path: "fixture.mov".into(),
            identity: MediaIdentity {
                algorithm: HashAlgorithm::Sha256,
                digest: "0".repeat(64),
            },
            video_stream: None,
        },
    }
}
