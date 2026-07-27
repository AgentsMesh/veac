use tempfile::tempdir;
use veac_artifact::*;

use super::support::{canonical_project, FakeEnvironment, GENERATED_SOURCE, MEDIA_SOURCE};
use crate::arguments::SubstitutionPolicy::{Original, Prefer, Require};
use crate::environment::Environment;

#[test]
fn required_exact_proxy_becomes_the_physical_backend_input() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), b"original").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut environment = FakeEnvironment::success();
    environment.observed =
        veac_runtime::asset::sha256_identity(&temp.path().join("clip.mp4")).unwrap();
    let prepared = crate::planning::prepare(&project, None, &environment).unwrap();
    let descriptor = proxy_descriptor(&prepared, &environment);
    let store = ArtifactStore::new(temp.path().join(".veac-artifacts"));
    let record = store.put(&descriptor, b"exact proxy").unwrap();
    let payload = store
        .open(&record.key)
        .unwrap()
        .unwrap()
        .payload_path()
        .to_owned();

    crate::commands::render(&project, None, None, None, Require, Original, &environment).unwrap();
    let calls = environment.executed.borrow();
    assert_eq!(calls.len(), 1);
    assert_ne!(input(&calls[0]), payload.to_str().unwrap());
    assert!(!calls[0].iter().any(|value| value.ends_with("clip.mp4")));
    assert_eq!(
        environment.consumed_inputs.borrow()[0],
        vec![b"exact proxy".to_vec()]
    );
}

#[test]
fn required_proxy_miss_fails_while_prefer_safely_falls_back_from_corruption() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), b"original").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut environment = FakeEnvironment::success();
    environment.observed =
        veac_runtime::asset::sha256_identity(&temp.path().join("clip.mp4")).unwrap();
    let error =
        crate::commands::render(&project, None, None, None, Require, Original, &environment)
            .unwrap_err();
    assert!(error.to_string().contains("PROXY_REQUIRED_MISSING"));
    assert!(environment.executed.borrow().is_empty());

    let prepared = crate::planning::prepare(&project, None, &environment).unwrap();
    let descriptor = proxy_descriptor(&prepared, &environment);
    let store = ArtifactStore::new(temp.path().join(".veac-artifacts"));
    let record = store.put(&descriptor, b"exact proxy").unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    std::fs::write(artifact.payload_path(), b"corrupt").unwrap();
    crate::commands::render(&project, None, None, None, Prefer, Original, &environment).unwrap();
    assert_eq!(environment.executed.borrow().len(), 1);
    assert_eq!(
        environment.consumed_inputs.borrow()[0],
        vec![b"original".to_vec()]
    );
}

#[test]
fn preferred_segment_is_stored_then_reused_and_corruption_never_falls_back() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let environment = FakeEnvironment::success();
    let prepared = crate::planning::prepare(&project, None, &environment).unwrap();
    let contract = segment_contract(&prepared, &environment);
    let store = ArtifactStore::new(temp.path().join(".veac-artifacts"));

    crate::commands::render(&project, None, None, None, Original, Prefer, &environment).unwrap();
    let segment = select_full_render_segment(&store, &contract)
        .unwrap()
        .expect("first render stores its exact full segment");
    crate::commands::render(&project, None, None, None, Original, Prefer, &environment).unwrap();
    let calls = environment.executed.borrow();
    assert_eq!(calls.len(), 2);
    assert!(calls[1].windows(2).any(|pair| pair == ["-c", "copy"]));
    assert_ne!(input(&calls[1]), segment.payload_path().to_str().unwrap());
    assert_eq!(
        environment.consumed_inputs.borrow()[1],
        vec![b"rendered".to_vec()]
    );
    drop(calls);

    std::fs::write(segment.payload_path(), b"corrupt").unwrap();
    let error = crate::commands::render(&project, None, None, None, Original, Prefer, &environment)
        .unwrap_err();
    assert!(error.to_string().contains("RENDER_SEGMENT_FAILED"));
    assert_eq!(environment.executed.borrow().len(), 2);
}

#[test]
fn required_segment_miss_starts_no_ffmpeg_task() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let environment = FakeEnvironment::success();
    let error =
        crate::commands::render(&project, None, None, None, Original, Require, &environment)
            .unwrap_err();
    assert!(error
        .to_string()
        .contains("RENDER_SEGMENT_REQUIRED_MISSING"));
    assert!(environment.executed.borrow().is_empty());
}

pub(super) fn proxy_descriptor(
    prepared: &crate::planning::PreparedPlan,
    environment: &dyn Environment,
) -> ArtifactDescriptor {
    let input = &prepared.plan.inputs[0];
    let stream = input.video.as_ref().unwrap().selection;
    MediaArtifactRequest {
        source_identity: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: input.observed_identity.digest.clone(),
        },
        producer: producer(environment),
        spec: MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream: stream,
            source_clock: SourceClockSpec::Identity {
                duration: normalized_time(
                    input
                        .video
                        .as_ref()
                        .and_then(|value| value.duration)
                        .or_else(|| {
                            input
                                .probe
                                .as_ref()
                                .and_then(|value| value.container_duration)
                        })
                        .unwrap(),
                    prepared.plan.header.source.timebase,
                ),
            },
            width: prepared.plan.output.width,
            height: prepared.plan.output.height,
            frame_rate: prepared.plan.output.frame_rate,
            crf: 28,
        }),
    }
    .descriptor()
    .unwrap()
}

fn normalized_time(value: veac_ir::RationalTime, timescale: u32) -> veac_ir::RationalTime {
    let numerator = i128::from(value.value) * i128::from(timescale);
    assert_eq!(numerator % i128::from(value.timescale), 0);
    veac_ir::RationalTime::new(
        i64::try_from(numerator / i128::from(value.timescale)).unwrap(),
        timescale,
    )
    .unwrap()
}

pub(super) fn segment_contract(
    prepared: &crate::planning::PreparedPlan,
    environment: &dyn Environment,
) -> FullRenderSegmentContract {
    FullRenderSegmentContract::new(
        &prepared.plan,
        prepared.bindings.input_substitution_proof(),
        producer(environment),
    )
    .unwrap()
}

pub(super) fn producer(environment: &dyn Environment) -> ProducerFingerprint {
    let value = environment.ffmpeg_fingerprint().unwrap();
    veac_runtime::workflow::media_artifact_producer(&value).unwrap()
}

fn input(arguments: &[String]) -> &str {
    arguments
        .windows(2)
        .find(|pair| pair[0] == "-i")
        .map(|pair| pair[1].as_str())
        .unwrap()
}
