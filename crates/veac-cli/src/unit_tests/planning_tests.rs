use tempfile::tempdir;
use veac_ir::{DeliverableId, MaterialSource, RenderConfigId};

use super::support::{canonical_project, FakeEnvironment, GENERATED_SOURCE, MEDIA_SOURCE};

#[test]
fn planning_selects_the_only_config_and_binds_inputs() {
    let temp = tempdir().unwrap();
    let generated = canonical_project(&temp, GENERATED_SOURCE);
    let prepared = crate::planning::prepare(&generated, None, &FakeEnvironment::success()).unwrap();
    assert_eq!(prepared.plan.output.render_config_id.as_str(), "out_main");
    assert!(prepared.bindings.inputs().is_empty());

    std::fs::write(temp.path().join("clip.mp4"), "fixture").unwrap();
    let media = canonical_project(&temp, MEDIA_SOURCE);
    let prepared =
        crate::planning::prepare(&media, Some("out_main"), &FakeEnvironment::success()).unwrap();
    assert_eq!(prepared.plan.inputs.len(), 1);
    assert_eq!(prepared.bindings.inputs().len(), 1);
}

#[test]
fn input_binding_defenses_reject_incomplete_resolved_inputs() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), "fixture").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let prepared = crate::planning::prepare(&project, None, &FakeEnvironment::success()).unwrap();

    let mut missing_material = prepared.plan.clone();
    missing_material.inputs[0].material_id = None;
    assert!(
        crate::planning::input_bindings(&missing_material, &prepared.material_paths)
            .unwrap_err()
            .to_string()
            .contains("INPUT_MATERIAL_MISSING")
    );
    assert!(
        crate::planning::input_bindings(&prepared.plan, &Default::default())
            .unwrap_err()
            .to_string()
            .contains("INPUT_BINDING_MISSING")
    );
}

#[test]
fn planning_requires_and_validates_render_config_selection() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    let mut second = envelope.project.render_configs[0].clone();
    second.id = RenderConfigId::new("out_second").unwrap();
    second.deliverables[0].id = DeliverableId::new("dlv_second").unwrap();
    second.deliverables[0].file_name = "second.mp4".into();
    envelope.project.render_configs.push(second);
    write_envelope(&project, &envelope);
    assert!(
        crate::planning::prepare(&project, None, &FakeEnvironment::success())
            .unwrap_err()
            .to_string()
            .contains("RENDER_CONFIG_REQUIRED")
    );
    assert_eq!(
        crate::planning::prepare(&project, Some("out_second"), &FakeEnvironment::success())
            .unwrap()
            .plan
            .output
            .deliverables[0]
            .file_name,
        "second.mp4"
    );
    assert!(
        crate::planning::prepare(&project, Some("bad"), &FakeEnvironment::success())
            .unwrap_err()
            .to_string()
            .contains("INVALID_RENDER_CONFIG_ID")
    );
    assert!(
        crate::planning::prepare(&project, Some("out_absent"), &FakeEnvironment::success())
            .unwrap_err()
            .to_string()
            .contains("RENDER_CONFIG_NOT_FOUND")
    );
}

#[test]
fn planning_reports_projects_without_outputs() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    envelope.project.render_configs.clear();
    write_envelope(&project, &envelope);
    assert!(
        crate::planning::prepare(&project, None, &FakeEnvironment::success())
            .unwrap_err()
            .to_string()
            .contains("RENDER_CONFIG_MISSING")
    );
}

#[test]
fn resolution_diagnostics_preserve_all_repairs() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), "fixture").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut envelope = crate::canonical::load(&project).unwrap();
    envelope.project.materials[0].source = MaterialSource::Remote {
        uri: "https://example.test/clip.mp4".into(),
    };
    let errors =
        veac_plan::resolve_one(&envelope, &RenderConfigId::new("out_main").unwrap()).unwrap_err();
    let message = crate::diagnostic::resolution(errors).to_string();
    assert!(message.contains("REMOTE_MATERIAL_UNRESOLVED"));
    assert!(message.contains("object: med_footage"));
    assert!(message.contains("/project/materials/med_footage/source"));
    assert!(message.contains("help:"));
}

fn write_envelope(path: &std::path::Path, envelope: &veac_ir::ProjectEnvelope) {
    std::fs::write(path, veac_ir::canonical_json(envelope).unwrap()).unwrap();
}
