use tempfile::tempdir;
use veac_ir::{DeliverableKind, ImageFormat, ImageSequenceOutput};

use super::super::support::{
    add_second_video, canonical_project, render, FakeEnvironment, GENERATED_SOURCE,
};

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
    let id = envelope.project.render_configs[0].deliverables[0]
        .id
        .clone();
    let video = envelope.project.render_configs[0]
        .video_deliverable_mut(&id)
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
