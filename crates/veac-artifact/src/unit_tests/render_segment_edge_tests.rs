use std::collections::BTreeMap;

use veac_ir::{
    CaptionSidecarFormat, CaptionSidecarOutput, Deliverable, DeliverableId, DeliverableKind,
    DeliverableTarget, SequenceId,
};

use crate::{test_support, *};

#[test]
fn full_segment_rejects_non_entry_multi_and_non_video_outputs() {
    let base = test_support::plan(b"source");
    let proof = originals(&base).input_substitution_proof();

    let mut wrong_sequence = base.clone();
    wrong_sequence.output.sequence_id = SequenceId::new("seq_other").unwrap();
    assert_invalid(&wrong_sequence, proof.clone());

    let mut multiple = base.clone();
    let mut second = multiple.output.deliverables[0].clone();
    second.id = DeliverableId::new("dlv_second").unwrap();
    second.target = DeliverableTarget::File {
        name: "second.mp4".into(),
    };
    multiple.output.deliverables.push(second);
    assert_invalid(&multiple, proof.clone());

    let mut mixed = base.clone();
    mixed.output.deliverables.push(Deliverable {
        id: DeliverableId::new("dlv_sidecar").unwrap(),
        target: DeliverableTarget::File {
            name: "captions.srt".into(),
        },
        kind: DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
            format: CaptionSidecarFormat::Srt,
            track_ids: vec![],
        }),
    });
    assert_invalid(&mixed, proof.clone());

    let mut non_video = base;
    non_video.output.deliverables[0].kind = DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
        format: CaptionSidecarFormat::Srt,
        track_ids: vec![],
    });
    assert_invalid(&non_video, proof);
}

#[test]
fn full_segment_rejects_missing_or_invalid_entry_sequence_ranges() {
    let base = test_support::plan(b"source");
    let proof = originals(&base).input_substitution_proof();

    let mut missing = base.clone();
    let absent = SequenceId::new("seq_absent").unwrap();
    missing.entry_sequence_id = absent.clone();
    missing.output.sequence_id = absent;
    assert_invalid(&missing, proof.clone());

    let mut invalid_duration = base;
    invalid_duration.sequences[0].duration.timescale = 0;
    let error = FullRenderSegmentContract::new(&invalid_duration, proof, test_support::producer())
        .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::InvalidContract);
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn full_segment_rejects_invalid_clock_digest_and_unhashable_plan() {
    let plan = test_support::plan(b"source");
    let invalid = ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: "bad".into(),
    };
    assert_invalid(&plan, invalid);

    let mut unhashable = plan;
    unhashable.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .opacity = veac_ir::Animatable::constant(f64::NAN);
    let error = FullRenderSegmentContract::new(
        &unhashable,
        ContentDigest::sha256(b"clocks"),
        test_support::producer(),
    )
    .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::InvalidContract);
}

#[test]
fn full_segment_accessors_miss_and_binding_contract_are_exact() {
    let plan = test_support::plan(b"source");
    let mut bindings = originals(&plan);
    let contract = FullRenderSegmentContract::new(
        &plan,
        bindings.input_substitution_proof(),
        test_support::producer(),
    )
    .unwrap();
    assert_eq!(contract.deliverable_id(), &plan.output.deliverables[0].id);
    let DeliverableKind::Video(video) = &plan.output.deliverables[0].kind else {
        unreachable!()
    };
    assert_eq!(contract.has_audio(), video.audio.is_some());
    let profile = contract.media_profile();
    let raster = plan.output.raster.as_ref().unwrap();
    assert_eq!(profile.width(), raster.width);
    assert_eq!(profile.height(), raster.height);
    assert_eq!(profile.frame_rate(), raster.frame_rate);
    assert_eq!(profile.deliverable(), &plan.output.deliverables[0]);

    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    assert!(select_full_render_segment(&store, &contract)
        .unwrap()
        .is_none());
    let record = store.put(contract.descriptor(), b"segment").unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    bindings
        .bind_verified_render_segment(&plan, &contract, &artifact)
        .unwrap();
    assert_eq!(
        bindings.full_render_segment().unwrap().contract(),
        &contract
    );
}

#[test]
fn full_segment_binding_rejects_a_different_verified_descriptor() {
    let plan = test_support::plan(b"source");
    let mut bindings = originals(&plan);
    let contract = FullRenderSegmentContract::new(
        &plan,
        bindings.input_substitution_proof(),
        test_support::producer(),
    )
    .unwrap();
    let mut other = contract.descriptor().clone();
    other.producer.version = "different".into();
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let record = store.put(&other, b"other segment").unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    assert_eq!(
        bindings
            .bind_verified_render_segment(&plan, &contract, &artifact)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::InvalidContract
    );
}

fn originals(plan: &veac_plan::ResolvedRenderPlan) -> ExecutionBindings {
    let paths = plan
        .inputs
        .iter()
        .map(|input| (input.id.clone(), "/media/source.mp4".into()))
        .collect::<BTreeMap<_, _>>();
    ExecutionBindings::from_originals(plan, &paths).unwrap()
}

fn assert_invalid(plan: &veac_plan::ResolvedRenderPlan, clocks: ContentDigest) {
    assert_eq!(
        FullRenderSegmentContract::new(plan, clocks, test_support::producer())
            .unwrap_err()
            .kind,
        ArtifactErrorKind::InvalidContract
    );
}
