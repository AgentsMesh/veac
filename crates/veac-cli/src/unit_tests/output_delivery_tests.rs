use tempfile::tempdir;
use veac_ir::{
    Deliverable, DeliverableId, DeliverableKind, ImageFormat, ImageSequenceOutput, MaterialSource,
};

use super::support::{
    add_second_video, canonical_project, render, FakeEnvironment, GENERATED_SOURCE, MEDIA_SOURCE,
};

#[test]
fn multiple_deliverables_bind_to_defaults_or_one_existing_directory() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    add_second_video(&project);
    let environment = FakeEnvironment::success();
    let mut prepared = crate::planning::prepare(&project, None, &environment).unwrap();
    let base = std::fs::canonicalize(temp.path()).unwrap();
    assert_eq!(
        crate::output::bind_render_outputs(&mut prepared, None).unwrap(),
        vec![base.join("render.mp4"), base.join("second.mp4")]
    );
    assert_eq!(prepared.bindings.outputs().len(), 2);

    let directory = temp.path().join("deliveries");
    std::fs::create_dir(&directory).unwrap();
    let directory = std::fs::canonicalize(directory).unwrap();
    assert_eq!(
        crate::output::bind_render_outputs(&mut prepared, Some(&directory)).unwrap(),
        vec![directory.join("render.mp4"), directory.join("second.mp4")]
    );
}

#[test]
fn multiple_deliverables_reject_file_or_missing_directory_override() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    add_second_video(&project);
    let mut prepared =
        crate::planning::prepare(&project, None, &FakeEnvironment::success()).unwrap();
    for path in [temp.path().join("outputs.mp4"), temp.path().join("missing")] {
        let error = crate::output::bind_render_outputs(&mut prepared, Some(&path)).unwrap_err();
        assert!(error.to_string().contains("OUTPUT_DIRECTORY_REQUIRED"));
    }
}

#[test]
fn image_pattern_cannot_consume_a_material_path() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip-1.png"), b"source").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    envelope.project.materials[0].source = MaterialSource::File {
        uri: "clip-1.png".to_owned(),
    };
    envelope.project.render_configs[0]
        .deliverables
        .push(Deliverable {
            id: DeliverableId::new("dlv_sequence").unwrap(),
            target: veac_ir::DeliverableTarget::ImageSequence {
                pattern: "clip-%d.png".to_owned(),
            },
            kind: DeliverableKind::ImageSequence(ImageSequenceOutput {
                format: ImageFormat::Png,
                start_number: 1,
            }),
        });
    write_project(&project, &envelope);
    let mut prepared =
        crate::planning::prepare(&project, None, &FakeEnvironment::success()).unwrap();
    let error = crate::output::bind_render_outputs(&mut prepared, None).unwrap_err();
    assert!(error.to_string().contains("OUTPUT_OVERWRITES_INPUT"));
}

#[test]
fn render_destination_cannot_be_the_checkpoint_store() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let store = temp.path().join(".veac-artifacts");
    std::fs::create_dir(&store).unwrap();
    let mut prepared =
        crate::planning::prepare(&project, None, &FakeEnvironment::success()).unwrap();
    let error = crate::output::bind_render_outputs(&mut prepared, Some(&store)).unwrap_err();
    assert!(error.to_string().contains("OUTPUT_RESERVED_DIRECTORY"));
}

#[test]
fn render_executes_every_deliverable_then_reuses_every_checkpoint() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    add_second_video(&project);
    let environment = FakeEnvironment::success();

    render(&project, Some(temp.path()), &environment).unwrap();
    assert_eq!(environment.executed.borrow().len(), 2);
    assert_eq!(
        std::fs::read(temp.path().join("render.mp4")).unwrap(),
        b"rendered"
    );
    assert_eq!(
        std::fs::read(temp.path().join("second.mp4")).unwrap(),
        b"rendered"
    );
    render(&project, Some(temp.path()), &environment).unwrap();
    assert_eq!(environment.executed.borrow().len(), 2);
}

#[test]
fn unavailable_required_encoder_starts_no_render_task() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let environment = FakeEnvironment {
        encoders: Default::default(),
        ..FakeEnvironment::success()
    };
    let error = render(&project, None, &environment).unwrap_err();
    assert!(error.to_string().contains("libx264"));
    assert!(environment.executed.borrow().is_empty());
    assert!(!temp.path().join("render.mp4").exists());
}

#[test]
fn two_pass_render_executes_analysis_before_delivery() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    let video = envelope.project.render_configs[0]
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap();
    video.pass_mode = veac_ir::PassMode::TwoPass;
    video.hardware = veac_ir::HardwareSelection::Software;
    video.video.rate_control = veac_ir::VideoRateControl::Bitrate {
        target_bps: 1_000_000,
        max_bps: Some(1_500_000),
        buffer_size_bits: Some(2_000_000),
    };
    write_project(&project, &envelope);
    let environment = FakeEnvironment::success();
    render(&project, None, &environment).unwrap();
    assert_eq!(environment.executed.borrow().len(), 2);
    assert!(temp.path().join("render.mp4").is_file());
}

#[test]
fn image_sequence_delivery_expands_the_authored_pattern() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    let deliverable = &mut envelope.project.render_configs[0].deliverables[0];
    deliverable.target = veac_ir::DeliverableTarget::ImageSequence {
        pattern: "frame-%04d.png".into(),
    };
    deliverable.kind = DeliverableKind::ImageSequence(ImageSequenceOutput {
        format: ImageFormat::Png,
        start_number: 1,
    });
    write_project(&project, &envelope);
    render(&project, None, &FakeEnvironment::success()).unwrap();
    assert!(temp.path().join("frame-0001.png").is_file());
}

#[cfg(unix)]
#[test]
fn unsafe_destination_directories_start_no_render_task() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let real = temp.path().join("real");
    let link = temp.path().join("link");
    let store = temp.path().join(".veac-artifacts");
    std::fs::create_dir(&real).unwrap();
    std::fs::create_dir(&store).unwrap();
    symlink(&real, &link).unwrap();
    let environment = FakeEnvironment::success();
    for (destination, code) in [
        (&link, "OUTPUT_DIRECTORY_REQUIRED"),
        (&store, "OUTPUT_RESERVED_DIRECTORY"),
    ] {
        let error = render(&project, Some(destination), &environment).unwrap_err();
        assert!(error.to_string().contains(code));
    }
    assert!(environment.executed.borrow().is_empty());
}

fn write_project(path: &std::path::Path, value: &veac_ir::ProjectEnvelope) {
    std::fs::write(path, veac_ir::canonical_json(value).unwrap()).unwrap();
}
