use tempfile::tempdir;
use veac_ir::{
    Deliverable, DeliverableId, DeliverableKind, ImageFormat, ImageSequenceOutput, MaterialSource,
};

use super::super::support::{
    add_second_video, canonical_project, FakeEnvironment, GENERATED_SOURCE, MEDIA_SOURCE,
};

#[test]
fn multiple_deliverables_bind_to_defaults_or_one_existing_directory() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    add_second_video(&project);
    let environment = FakeEnvironment::success();
    let mut prepared =
        crate::planning::prepare_with_material_root(&project, None, None, &environment).unwrap();
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
    let mut prepared = crate::planning::prepare_with_material_root(
        &project,
        None,
        None,
        &FakeEnvironment::success(),
    )
    .unwrap();
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
    let mut prepared = crate::planning::prepare_with_material_root(
        &project,
        None,
        None,
        &FakeEnvironment::success(),
    )
    .unwrap();
    let error = crate::output::bind_render_outputs(&mut prepared, None).unwrap_err();
    assert!(error.to_string().contains("OUTPUT_OVERWRITES_INPUT"));
}

#[test]
fn render_destination_cannot_be_the_checkpoint_store() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let store = temp.path().join(".veac-artifacts");
    std::fs::create_dir(&store).unwrap();
    let mut prepared = crate::planning::prepare_with_material_root(
        &project,
        None,
        None,
        &FakeEnvironment::success(),
    )
    .unwrap();
    let error = crate::output::bind_render_outputs(&mut prepared, Some(&store)).unwrap_err();
    assert!(error.to_string().contains("OUTPUT_RESERVED_DIRECTORY"));
}

fn write_project(path: &std::path::Path, value: &veac_ir::ProjectEnvelope) {
    std::fs::write(path, veac_ir::canonical_json(value).unwrap()).unwrap();
}
