use std::collections::BTreeMap;

use crate::{test_support, *};

#[test]
fn full_segment_identity_binds_plan_profile_range_fidelity_and_source_clocks() {
    let plan = test_support::plan(b"source");
    let bindings = originals(&plan, "/media/source.mp4");
    let base = contract(&plan, bindings.input_substitution_proof());
    let roles: Vec<_> = base
        .descriptor()
        .dependencies
        .iter()
        .map(|value| value.role)
        .collect();
    assert_eq!(
        roles,
        [
            ArtifactDependencyRole::Plan,
            ArtifactDependencyRole::Profile,
            ArtifactDependencyRole::SourceClocks,
        ]
    );
    let ArtifactParameters::RenderSegment(parameters) = &base.descriptor().parameters else {
        unreachable!()
    };
    assert_eq!(
        parameters.fidelity,
        RenderSegmentFidelity::ExactDeliveryMaster
    );
    assert_eq!(base.range().start.value, 0);

    let mut profile = plan.clone();
    profile.output.raster.as_mut().unwrap().width += 2;
    let changed = contract(&profile, bindings.input_substitution_proof());
    assert_ne!(base.media_profile(), changed.media_profile());
    assert_ne!(key(&base), key(&changed));
    let mut rate = plan.clone();
    rate.output.raster.as_mut().unwrap().frame_rate = veac_ir::Rational::new(24, 1).unwrap();
    let rate = contract(&rate, bindings.input_substitution_proof());
    assert_ne!(base.media_profile(), rate.media_profile());
    let mut captions = plan.clone();
    captions.output.raster.as_mut().unwrap().captions = veac_ir::CaptionOutput::Discard;
    let captions = contract(&captions, bindings.input_substitution_proof());
    assert_ne!(base.media_profile(), captions.media_profile());
    let clocks = contract(&plan, ContentDigest::sha256(b"other clocks"));
    assert_ne!(key(&base), key(&clocks));
    let mut range = plan.clone();
    range.sequences.last_mut().unwrap().duration.value -= 1;
    assert_ne!(
        key(&base),
        key(&contract(&range, bindings.input_substitution_proof()))
    );
}

#[test]
fn full_segment_identity_binds_the_backend_producer() {
    let plan = test_support::plan(b"source");
    let clocks = originals(&plan, "/media/source.mp4").input_substitution_proof();
    let first = test_support::producer();
    let mut second = first.clone();
    second.configuration = ContentDigest::sha256(b"different backend build");
    let first = FullRenderSegmentContract::new(&plan, clocks.clone(), first).unwrap();
    let second = FullRenderSegmentContract::new(&plan, clocks, second).unwrap();
    assert_ne!(first.descriptor().producer, second.descriptor().producer);
    assert_ne!(key(&first), key(&second));
}

#[test]
fn exact_verified_segment_binds_and_drift_or_corruption_fails_closed() {
    let plan = test_support::plan(b"source");
    let mut bindings = originals(&plan, "/media/source.mp4");
    let input_proof = bindings.input_substitution_proof();
    let contract = contract(&plan, input_proof.clone());
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let record = store
        .put(contract.descriptor(), b"rendered segment")
        .unwrap();
    let artifact = select_full_render_segment(&store, &contract)
        .unwrap()
        .unwrap();
    bindings
        .bind_verified_render_segment(&plan, &contract, &artifact)
        .unwrap();
    let segment = bindings.full_render_segment().unwrap();
    assert_eq!(segment.resource().artifact_key(), Some(&record.key));
    assert_ne!(bindings.substitution_proof(), input_proof);

    let mut drifted = plan.clone();
    drifted.output.raster.as_mut().unwrap().height += 2;
    let mut wrong = originals(&drifted, "/media/source.mp4");
    assert_eq!(
        wrong
            .bind_verified_render_segment(&drifted, &contract, &artifact)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::InvalidContract
    );

    std::fs::write(artifact.payload_path(), b"corrupt").unwrap();
    assert_eq!(
        select_full_render_segment(&store, &contract)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::CorruptCache
    );
}

fn originals(plan: &veac_plan::ResolvedRenderPlan, path: &str) -> ExecutionBindings {
    let paths = plan
        .inputs
        .iter()
        .map(|input| (input.id.clone(), path.into()))
        .collect::<BTreeMap<_, _>>();
    ExecutionBindings::from_originals(plan, &paths).unwrap()
}

fn contract(
    plan: &veac_plan::ResolvedRenderPlan,
    clocks: ContentDigest,
) -> FullRenderSegmentContract {
    FullRenderSegmentContract::new(plan, clocks, test_support::producer()).unwrap()
}

fn key(contract: &FullRenderSegmentContract) -> ContentDigest {
    artifact_key(contract.descriptor()).unwrap()
}
