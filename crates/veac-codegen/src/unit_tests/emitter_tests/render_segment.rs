use veac_artifact::*;
use veac_codegen::emitter::emit_all;

use super::binding_routes::{av_plan, typed, video_command};
use super::support::{fixture, resolved};

#[test]
fn exact_full_segment_is_the_only_protected_input_and_skips_encoding() {
    let mut plan = resolved(&fixture());
    let video = plan.output.video_deliverable_mut().unwrap();
    video.pass_mode = veac_plan::canonical::PassMode::TwoPass;
    video.video.rate_control = veac_plan::canonical::VideoRateControl::Bitrate {
        target_bps: 1_000_000,
        max_bps: None,
        buffer_bps: None,
    };
    video.hardware = veac_plan::canonical::HardwareSelection::Software;
    let mut bindings = typed(&plan);
    let contract = contract(&plan, &bindings);
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let record = store.put(contract.descriptor(), b"exact master").unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    bindings
        .bind_verified_render_segment(&plan, &contract, &artifact)
        .unwrap();

    let bundle = emit_all(&plan, &bindings).unwrap();
    assert_eq!(bundle.tasks().len(), 1);
    assert_eq!(bundle.requirements().len(), 2);
    for (kind, name) in [("demuxer", "mov"), ("muxer", "mp4")] {
        assert!(bundle
            .requirements()
            .iter()
            .any(|value| value.kind().as_str() == kind && value.name() == name));
    }
    assert_eq!(bundle.protected_resources().len(), 1);
    assert_eq!(
        bundle.protected_resources()[0].path,
        artifact.payload_path()
    );
    let command = video_command(&bundle);
    assert_eq!(command.inputs.len(), 1);
    assert_eq!(command.inputs[0].path, artifact.payload_path());
    assert!(command.filter_graph.is_none());
    assert_eq!(command.maps, ["0:0"]);
    assert!(command
        .output_args
        .windows(2)
        .any(|pair| pair == ["-c", "copy"]));
    assert_eq!(bundle.substitution_proof(), &bindings.substitution_proof());
}

#[test]
fn profile_drift_cannot_bind_an_exact_segment() {
    let plan = resolved(&fixture());
    let bindings = typed(&plan);
    let contract = contract(&plan, &bindings);
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let record = store.put(contract.descriptor(), b"exact master").unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    let mut drifted = plan.clone();
    drifted.output.width += 2;
    let mut local = typed(&drifted);
    assert_eq!(
        local
            .bind_verified_render_segment(&drifted, &contract, &artifact)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::InvalidContract
    );
}

#[test]
fn exact_av_segment_maps_audio_and_keeps_mov_faststart_delivery_flags() {
    let mut plan = av_plan();
    plan.output.deliverables[0].file_name = "master.mov".to_owned();
    let video = plan.output.video_deliverable_mut().unwrap();
    video.container = veac_plan::canonical::OutputFormat::Mov;
    video.optimize_for_streaming = true;
    let mut bindings = typed(&plan);
    let contract = contract(&plan, &bindings);
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let record = store
        .put(contract.descriptor(), b"exact av master")
        .unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    bindings
        .bind_verified_render_segment(&plan, &contract, &artifact)
        .unwrap();
    let bundle = emit_all(&plan, &bindings).unwrap();
    let command = video_command(&bundle);
    assert_eq!(command.maps, ["0:0", "0:1"]);
    assert!(command
        .output_args
        .windows(2)
        .any(|pair| pair == ["-movflags", "+faststart"]));
    assert!(command
        .output_args
        .windows(2)
        .any(|pair| pair == ["-f", "mov"]));
}

#[test]
fn bound_segment_rejects_a_later_deliverable_identity_change() {
    let plan = resolved(&fixture());
    let mut bindings = typed(&plan);
    let contract = contract(&plan, &bindings);
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path());
    let record = store.put(contract.descriptor(), b"exact master").unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    bindings
        .bind_verified_render_segment(&plan, &contract, &artifact)
        .unwrap();

    let mut drifted = plan;
    drifted.output.deliverables[0].id =
        veac_plan::canonical::DeliverableId::new("dlv_drifted").unwrap();
    let error = emit_all(&drifted, &bindings).unwrap_err();
    assert_eq!(
        error.diagnostics()[0].code,
        "RENDER_SEGMENT_BINDING_INVALID"
    );
}

fn contract(
    plan: &veac_plan::ResolvedRenderPlan,
    bindings: &ExecutionBindings,
) -> FullRenderSegmentContract {
    FullRenderSegmentContract::new(
        plan,
        bindings.input_substitution_proof(),
        ProducerFingerprint {
            name: "ffmpeg-test".into(),
            version: "1".into(),
            configuration: ContentDigest::sha256(b"ffmpeg-test"),
        },
    )
    .unwrap()
}
