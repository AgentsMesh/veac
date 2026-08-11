use veac_artifact::{
    ArtifactStore, ContentDigest, ExecutionBindings, FullRenderSegmentContract, ProducerFingerprint,
};
use veac_plan::canonical::{
    DeliverableId, DeliverableKind, DeliverableTarget, ImageFormat, ImageSequenceOutput,
    VideoCadence,
};

use super::support::*;

#[test]
fn full_segment_skips_only_its_video_graph() {
    let mut video = resolved(&fixture());
    reverse(&mut video);
    video.inputs[0].video.as_mut().unwrap().info.cadence = VideoCadence::Variable;
    assert_eq!(codes(&video), ["PLAN_REVERSE_CADENCE_UNSUPPORTED"]);

    let segment = segment_bindings(&video);
    assert!(codes_with(&video, &segment).is_empty());

    let mut image = video;
    image.output.deliverables = vec![image_delivery()];
    assert_eq!(codes(&image), ["PLAN_REVERSE_CADENCE_UNSUPPORTED"]);
}

fn segment_bindings(plan: &veac_plan::ResolvedRenderPlan) -> ExecutionBindings {
    let mut bindings = crate::unit_tests::emitter_tests::support::bindings(plan);
    let producer = ProducerFingerprint {
        name: "reverse-segment-test".into(),
        version: "1".into(),
        configuration: ContentDigest::sha256(b"segment"),
    };
    let contract =
        FullRenderSegmentContract::new(plan, bindings.input_substitution_proof(), producer)
            .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let record = store.put(contract.descriptor(), b"segment").unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    bindings
        .bind_verified_render_segment(plan, &contract, &artifact)
        .unwrap();
    bindings
}

fn image_delivery() -> veac_plan::canonical::Deliverable {
    veac_plan::canonical::Deliverable {
        id: DeliverableId::new("dlv_reverse_frames").unwrap(),
        target: DeliverableTarget::ImageSequence {
            pattern: "reverse-%04d.png".into(),
        },
        kind: DeliverableKind::ImageSequence(ImageSequenceOutput {
            format: ImageFormat::Png,
            start_number: 0,
        }),
    }
}
