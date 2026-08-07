use std::collections::BTreeMap;

use veac_ir::{RationalTime, TimeRange};

use crate::{test_support, *};

#[test]
fn source_clock_maps_nonzero_ranges_and_enforces_half_open_points() {
    let range = TimeRange::new(time(50), time(30)).unwrap();
    let clock = SourceClock::bounded(range, time(10)).unwrap();
    assert_eq!(clock.map_point(time(50)), Some(time(10)));
    assert_eq!(clock.map_point(time(65)), Some(time(25)));
    assert_eq!(clock.map_point(time(80)), None);
    assert_eq!(clock.map_boundary(time(80)), Some(time(40)));
    assert!(clock.covers_range(TimeRange::new(time(55), time(25)).unwrap()));
    assert!(!clock.covers_range(TimeRange::new(time(40), time(20)).unwrap()));
    assert_eq!(
        SourceClock::bounded(
            TimeRange {
                start: time(0),
                duration: RationalTime::zero(10).unwrap(),
            },
            time(0),
        )
        .unwrap_err()
        .kind,
        ArtifactErrorKind::InvalidContract
    );
}

#[test]
fn original_bindings_preserve_identity_paths_and_streams() {
    let plan = test_support::plan(b"source");
    let mut paths = BTreeMap::new();
    for input in &plan.inputs {
        paths.insert(input.id.clone(), format!("/media/{}.mov", input.id).into());
    }
    let mut bindings = ExecutionBindings::from_originals(&plan, &paths).unwrap();
    for input in &plan.inputs {
        let binding = bindings.input(&input.id).unwrap();
        assert_eq!(
            binding.resource().unwrap().identity(),
            &input.observed_identity
        );
        assert_eq!(binding.video().is_some(), input.video.is_some());
        assert_eq!(binding.audio().is_some(), input.audio.is_some());
        assert_eq!(
            binding.resource().unwrap().path(),
            paths.get(&input.id).unwrap().as_path()
        );
        assert_eq!(binding.resource().unwrap().artifact_kind(), None);
    }
    let deliverable = plan.output.deliverables[0].id.clone();
    bindings
        .bind_output(deliverable.clone(), "/output/master.mp4".into())
        .unwrap();
    assert_eq!(
        bindings.output(&deliverable).unwrap(),
        std::path::Path::new("/output/master.mp4")
    );
}

#[test]
fn only_a_verified_source_bound_proxy_can_issue_artifact_provenance() {
    let plan = test_support::plan(b"source");
    let input = &plan.inputs[0];
    let mut paths = BTreeMap::new();
    paths.insert(input.id.clone(), "/media/original.mov".into());
    let mut bindings = ExecutionBindings::from_originals(&plan, &paths).unwrap();
    let original_proof = bindings.substitution_proof();
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let descriptor = proxy_descriptor(input, ArtifactKind::ProxyVideo);
    let record = store.put(&descriptor, b"video proxy").unwrap();
    let proxy = store
        .open_verified(&record.key, &descriptor)
        .unwrap()
        .unwrap();
    bindings
        .bind_verified_proxy(input, MediaRole::Video, &proxy)
        .unwrap();
    let stream = bindings.input(&input.id).unwrap().video().unwrap();
    assert_eq!(
        stream.resource().provenance_kind(),
        BindingProvenanceKind::Artifact
    );
    assert_eq!(stream.resource().artifact_key(), Some(&record.key));
    assert_eq!(
        stream.resource().artifact_kind(),
        Some(ArtifactKind::ProxyVideo)
    );
    assert_eq!(stream.resource().identity().digest, record.content.value);
    assert_eq!(
        stream.clock().logical_range(),
        Some(TimeRange::new(time(60), time(60)).unwrap())
    );
    assert_eq!(stream.clock().map_point(time(60)), Some(time(0)));
    assert_ne!(bindings.substitution_proof(), original_proof);
}

#[test]
fn verified_proxy_rejects_the_wrong_role_and_source_identity() {
    let plan = test_support::plan(b"source");
    let input = &plan.inputs[0];
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let mut wrong_source = proxy_descriptor(input, ArtifactKind::ProxyVideo);
    wrong_source.dependencies[0].identity = ContentDigest::sha256(b"wrong source");
    for descriptor in [
        proxy_descriptor(input, ArtifactKind::ProxyAudio),
        wrong_source,
    ] {
        let record = store
            .put(&descriptor, descriptor.kind_name().as_bytes())
            .unwrap();
        let artifact = store.open(&record.key).unwrap().unwrap();
        let mut bindings = ExecutionBindings::default();
        assert_eq!(
            bindings
                .bind_verified_proxy(input, MediaRole::Video, &artifact)
                .unwrap_err()
                .kind,
            ArtifactErrorKind::InvalidContract
        );
    }
}

fn proxy_descriptor(input: &veac_plan::ResolvedInput, kind: ArtifactKind) -> ArtifactDescriptor {
    let stream = input.video.as_ref().unwrap().selection;
    let clock = SourceClockSpec::Bounded {
        logical_range: TimeRange::new(time(60), time(60)).unwrap(),
    };
    let spec = match kind {
        ArtifactKind::ProxyVideo => MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream: stream,
            source_clock: clock,
            width: 640,
            height: 360,
            frame_rate: veac_ir::Rational::new(30, 1).unwrap(),
            crf: 24,
        }),
        ArtifactKind::ProxyAudio => MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: stream,
            source_clock: clock,
            sample_rate: 48_000,
            channels: 2,
        }),
        _ => unreachable!(),
    };
    let parameters = match spec {
        MediaArtifactSpec::ProxyVideo(value) => ArtifactParameters::ProxyVideo(value),
        MediaArtifactSpec::ProxyAudio(value) => ArtifactParameters::ProxyAudio(value),
        _ => unreachable!(),
    };
    ArtifactDescriptor::new(
        test_support::producer(),
        vec![ArtifactDependency::new(
            ArtifactDependencyRole::Input,
            ContentDigest {
                algorithm: DigestAlgorithm::Sha256,
                value: input.observed_identity.digest.clone(),
            },
        )],
        parameters,
    )
}

trait KindName {
    fn kind_name(&self) -> &'static str;
}

impl KindName for ArtifactDescriptor {
    fn kind_name(&self) -> &'static str {
        match self.kind() {
            ArtifactKind::ProxyVideo => "video",
            ArtifactKind::ProxyAudio => "audio",
            _ => "other",
        }
    }
}

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 10).unwrap()
}
