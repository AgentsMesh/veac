use veac_ir::{HashAlgorithm, RationalTime};

use crate::{
    execution_bindings_edge_tests::{add_audio, original_paths},
    test_support, *,
};

#[path = "proxy_selection_tests/support.rs"]
mod support;
use support::*;

#[test]
fn complete_proxy_selection_binds_independent_video_and_audio_artifacts() {
    let mut plan = test_support::plan(b"source");
    add_audio(&mut plan);
    let input = &plan.inputs[0];
    let video = proxy_descriptor(input, MediaRole::Video, video_spec(input));
    let audio = proxy_descriptor(input, MediaRole::Audio, audio_spec(input));
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    store.put(&video, b"video proxy").unwrap();
    store.put(&audio, b"audio proxy").unwrap();
    let selection = select_proxy(
        &store,
        &ProxySelectionRequest {
            source_identity: source(input),
            video: Some(video),
            audio: Some(audio),
        },
    )
    .unwrap();
    let mut bindings = ExecutionBindings::from_originals(&plan, &original_paths(&plan)).unwrap();
    bindings.bind_proxy_selection(input, &selection).unwrap();
    let bound = bindings.input(&input.id).unwrap();
    let video = bound.video().unwrap();
    let audio = bound.audio().unwrap();
    assert_eq!(video.physical_stream(), stream(0, 0));
    assert_eq!(audio.physical_stream(), stream(0, 0));
    assert_eq!(video.clock().logical_range(), Some(range(0, 60)));
    assert_eq!(audio.clock().logical_range(), Some(range(0, 600)));
    assert_eq!(
        audio.clock().physical_start(),
        RationalTime::zero(600).unwrap()
    );
    assert_ne!(video.resource().path(), audio.resource().path());
}

#[test]
fn partial_selection_binds_only_the_present_media_role() {
    let mut plan = test_support::plan(b"source");
    add_audio(&mut plan);
    let input = &plan.inputs[0];
    let audio = proxy_descriptor(input, MediaRole::Audio, audio_spec(input));
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    store.put(&audio, b"audio proxy").unwrap();
    let selection = select_proxy(
        &store,
        &ProxySelectionRequest {
            source_identity: source(input),
            video: None,
            audio: Some(audio),
        },
    )
    .unwrap();
    let mut bindings = ExecutionBindings::default();
    bindings.bind_proxy_selection(input, &selection).unwrap();
    let bound = bindings.input(&input.id).unwrap();
    assert!(bound.resource().is_none());
    assert!(bound.video().is_none());
    assert!(bound.audio().is_some());
}

#[test]
fn selection_rejects_another_source_and_non_sha_input_identity() {
    let plan = test_support::plan(b"source");
    let input = &plan.inputs[0];
    let mut bindings = ExecutionBindings::default();
    let selection = ProxyBinding {
        source_identity: ContentDigest::sha256(b"other"),
        video: None,
        audio: None,
    };
    assert_invalid(bindings.bind_proxy_selection(input, &selection));

    let mut unsupported = input.clone();
    unsupported.observed_identity.algorithm = HashAlgorithm::Blake3;
    assert_invalid(bindings.bind_proxy_selection(&unsupported, &selection));
    unsupported.observed_identity.algorithm = HashAlgorithm::Sha256;
    unsupported.observed_identity.digest = "bad".into();
    assert_invalid(bindings.bind_proxy_selection(&unsupported, &selection));
}

#[test]
fn verified_proxy_rejects_invalid_parameters_role_stream_and_clock() {
    let plan = test_support::plan(b"source");
    let input = &plan.inputs[0];
    let mut wrong_dependency = proxy_descriptor(input, MediaRole::Video, video_spec(input));
    wrong_dependency.dependencies[0].role = ArtifactDependencyRole::Source;
    let cases = [
        wrong_dependency,
        proxy_descriptor(
            input,
            MediaRole::Video,
            audio_spec_for_stream(input.video.as_ref().unwrap().selection),
        ),
        proxy_descriptor(
            input,
            MediaRole::Video,
            video_spec_for_stream(
                stream(9, 9),
                SourceClockSpec::Identity {
                    duration: RationalTime::new(600, 600).unwrap(),
                },
            ),
        ),
    ];
    for (index, descriptor) in cases.into_iter().enumerate() {
        let temp = tempfile::tempdir().unwrap();
        let store = ArtifactStore::new(temp.path());
        let record = store
            .put(&descriptor, format!("proxy-{index}").as_bytes())
            .unwrap();
        let artifact = store.open(&record.key).unwrap().unwrap();
        assert_invalid(ExecutionBindings::default().bind_verified_proxy(
            input,
            MediaRole::Video,
            &artifact,
        ));
    }
    let invalid_clock = proxy_descriptor(
        input,
        MediaRole::Video,
        video_spec_for_stream(
            input.video.as_ref().unwrap().selection,
            SourceClockSpec::Identity {
                duration: RationalTime {
                    value: 1,
                    timescale: 0,
                },
            },
        ),
    );
    assert_invalid(invalid_clock.validate());
}

#[test]
fn proxy_role_absence_is_rejected_before_a_binding_is_created() {
    let mut plan = test_support::plan(b"source");
    let input = &mut plan.inputs[0];
    input.video = None;
    let spec = video_spec_for_stream(
        stream(0, 0),
        SourceClockSpec::Identity {
            duration: RationalTime::new(600, 600).unwrap(),
        },
    );
    let descriptor = proxy_descriptor(input, MediaRole::Video, spec);
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let record = store.put(&descriptor, b"proxy").unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    assert_invalid(ExecutionBindings::default().bind_verified_proxy(
        input,
        MediaRole::Video,
        &artifact,
    ));
}
