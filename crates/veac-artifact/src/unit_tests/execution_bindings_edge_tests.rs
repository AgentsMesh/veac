use std::collections::BTreeMap;

use veac_ir::{AudioStreamInfo, DeliverableId, StreamDisposition, StreamSelection};
use veac_plan::{PlanInputId, ResolvedAudioStream};

use crate::{test_support, *};

#[test]
fn originals_reject_missing_extra_and_empty_paths() {
    let plan = test_support::plan(b"source");
    assert_eq!(
        ExecutionBindings::from_originals(&plan, &BTreeMap::new())
            .unwrap_err()
            .kind,
        ArtifactErrorKind::MissingBinding
    );

    let mut paths = original_paths(&plan);
    paths.insert(
        PlanInputId::new("pin_extra").unwrap(),
        "/media/extra.mov".into(),
    );
    assert_eq!(
        ExecutionBindings::from_originals(&plan, &paths)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::InvalidContract
    );

    let mut bindings = ExecutionBindings::default();
    assert_eq!(
        bindings
            .bind_original(&plan.inputs[0], "".into())
            .unwrap_err()
            .kind,
        ArtifactErrorKind::InvalidContract
    );
    assert_eq!(
        bindings
            .bind_output(plan.output.deliverables[0].id.clone(), "".into())
            .unwrap_err()
            .kind,
        ArtifactErrorKind::InvalidContract
    );
}

#[test]
fn typed_bindings_expose_identity_aware_inputs_and_outputs() {
    let plan = test_support::plan(b"source");
    let output_id = plan.output.deliverables[0].id.clone();
    let mut bindings = ExecutionBindings::from_originals(&plan, &original_paths(&plan)).unwrap();
    bindings
        .bind_output(output_id.clone(), "/output/master.mp4".into())
        .unwrap();
    let input = &plan.inputs[0];
    let bound = bindings.input(&input.id).unwrap();
    let resource = bound.resource().unwrap();
    assert_eq!(bindings.inputs().len(), 1);
    assert_eq!(bindings.outputs().len(), 1);
    assert_eq!(
        bindings.output(&output_id),
        Some(std::path::Path::new("/output/master.mp4"))
    );
    assert_eq!(resource.path(), std::path::Path::new("/media/source.mov"));
    assert_eq!(resource.provenance_kind(), BindingProvenanceKind::Original);
    assert_eq!(resource.artifact_key(), None);
    assert_eq!(bound.source_identity(), Some(&input.observed_identity));
    assert_eq!(
        bound.source_stream(MediaRole::Video),
        input.video.as_ref().map(|stream| stream.selection)
    );
    assert_eq!(bound.source_stream(MediaRole::Audio), None);
    assert_eq!(bound.stream(MediaRole::Audio), None);
    let video = bound.stream(MediaRole::Video).unwrap();
    assert_eq!(
        video.physical_stream(),
        input.video.as_ref().unwrap().selection
    );
    assert_eq!(video.clock().timescale(), 600);
}

#[test]
fn typed_bindings_allow_partial_and_resource_only_bindings() {
    let plan = test_support::plan(b"source");
    let empty = ExecutionBindings::default();
    assert!(empty.inputs().is_empty());

    let mut input = plan.inputs[0].clone();
    input.video = None;
    input.audio = None;
    let mut bindings = ExecutionBindings::default();
    bindings
        .bind_original(&input, "/assets/font-or-resource".into())
        .unwrap();
    let bound = bindings.input(&input.id).unwrap();
    assert!(bound.resource().is_some());
    assert!(bound.video().is_none());
    assert!(bound.audio().is_none());
}

#[test]
fn original_audio_stream_uses_the_selected_physical_stream() {
    let mut plan = test_support::plan(b"source");
    add_audio(&mut plan);
    let bindings = ExecutionBindings::from_originals(&plan, &original_paths(&plan)).unwrap();
    let input = &plan.inputs[0];
    let audio = bindings.input(&input.id).unwrap().audio().unwrap();
    assert_eq!(
        audio.physical_stream(),
        input.audio.as_ref().unwrap().selection
    );
    assert_eq!(
        audio.resource().provenance_kind(),
        BindingProvenanceKind::Original
    );
}

#[test]
fn missing_output_accessor_is_none() {
    let bindings = ExecutionBindings::default();
    assert_eq!(
        bindings.output(&DeliverableId::new("dlv_missing").unwrap()),
        None
    );
}

pub(super) fn add_audio(plan: &mut veac_plan::ResolvedRenderPlan) {
    plan.inputs[0].audio = Some(ResolvedAudioStream {
        selection: StreamSelection {
            global_index: 1,
            type_index: 0,
        },
        codec: "aac".into(),
        start_time: None,
        duration: plan.inputs[0]
            .probe
            .as_ref()
            .and_then(|probe| probe.container_duration),
        disposition: StreamDisposition {
            default: true,
            attached_picture: false,
            timed_thumbnail: false,
        },
        info: AudioStreamInfo {
            sample_rate: 48_000,
            channels: 2,
            channel_layout: "stereo".into(),
        },
    });
}

pub(super) fn original_paths(
    plan: &veac_plan::ResolvedRenderPlan,
) -> BTreeMap<PlanInputId, std::path::PathBuf> {
    plan.inputs
        .iter()
        .map(|input| (input.id.clone(), "/media/source.mov".into()))
        .collect()
}
