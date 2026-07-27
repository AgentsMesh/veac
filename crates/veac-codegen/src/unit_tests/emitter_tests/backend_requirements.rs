use veac_artifact::{
    ArtifactDependency, ArtifactDescriptor, ArtifactKind, ArtifactStore, ContentDigest,
    DigestAlgorithm, ExecutionBindings, MediaArtifactSpec, MediaRole, ProducerFingerprint,
    ProxyVideoSpec, SourceClockSpec,
};
use veac_codegen::emitter::{emit_all, BackendCapabilityKind};
use veac_plan::canonical::{Rational, RationalTime};

use super::support::{bindings, fixture, resolved};

#[test]
fn generated_video_declares_exact_input_output_and_filter_capabilities() {
    let plan = resolved(&fixture());
    let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
    for (kind, name) in [
        (BackendCapabilityKind::Encoder, "libx264"),
        (BackendCapabilityKind::Decoder, "h264"),
        (BackendCapabilityKind::Muxer, "mp4"),
        (BackendCapabilityKind::Demuxer, "mov"),
        (BackendCapabilityKind::Filter, "trim"),
        (BackendCapabilityKind::Filter, "setpts"),
        (BackendCapabilityKind::Filter, "format"),
    ] {
        assert!(requires(&bundle, kind, name), "missing {kind:?} {name}");
    }
    assert!(!requires(&bundle, BackendCapabilityKind::Decoder, "aac"));
}

#[test]
fn verified_proxy_requirements_describe_proxy_not_original_media() {
    let mut plan = resolved(&fixture());
    let input = &mut plan.inputs[0];
    input.video.as_mut().unwrap().codec = "mpeg4".to_owned();
    input.probe.as_mut().unwrap().container_format = "avi".to_owned();
    let input = input.clone();
    let descriptor = proxy_descriptor(&input);
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let record = store.put(&descriptor, b"verified-proxy").unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    let mut bindings = ExecutionBindings::default();
    bindings
        .bind_verified_proxy(&input, MediaRole::Video, &artifact)
        .unwrap();
    bindings
        .bind_output(
            plan.output.deliverables[0].id.clone(),
            temp.path().join("master.mp4"),
        )
        .unwrap();

    let bundle = emit_all(&plan, &bindings).unwrap();
    assert!(requires(&bundle, BackendCapabilityKind::Decoder, "h264"));
    assert!(requires(&bundle, BackendCapabilityKind::Demuxer, "mov"));
    assert!(!requires(&bundle, BackendCapabilityKind::Decoder, "mpeg4"));
    assert!(!requires(&bundle, BackendCapabilityKind::Demuxer, "avi"));
}

fn proxy_descriptor(input: &veac_plan::ResolvedInput) -> ArtifactDescriptor {
    let spec = MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
        source_stream: input.video.as_ref().unwrap().selection,
        source_clock: SourceClockSpec::Identity {
            duration: RationalTime::new(600, 600).unwrap(),
        },
        width: 640,
        height: 360,
        frame_rate: Rational::new(30, 1).unwrap(),
        crf: 24,
    });
    ArtifactDescriptor::new(
        ArtifactKind::ProxyVideo,
        ProducerFingerprint {
            name: "veac-test".to_owned(),
            version: "1".to_owned(),
            configuration: ContentDigest::sha256(b"proxy-test"),
        },
        vec![ArtifactDependency {
            role: "input".to_owned(),
            identity: ContentDigest {
                algorithm: DigestAlgorithm::Sha256,
                value: input.observed_identity.digest.clone(),
            },
        }],
        serde_json::to_value(spec).unwrap(),
    )
}

fn requires(
    bundle: &veac_codegen::emitter::BackendBundle,
    kind: BackendCapabilityKind,
    name: &str,
) -> bool {
    bundle
        .requirements()
        .iter()
        .any(|value| value.kind() == kind && value.name() == name)
}
