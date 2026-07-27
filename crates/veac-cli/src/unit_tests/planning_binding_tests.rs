use tempfile::tempdir;

use super::support::{canonical_project, FakeEnvironment, MEDIA_SOURCE};

#[test]
fn planning_resolves_an_explicit_verified_binding_manifest() {
    let fixture = BindingFixture::new();
    let prepared = crate::planning::prepare_with_bindings(
        &fixture.project,
        None,
        Some(&fixture.bindings),
        &fixture.environment,
    )
    .unwrap();

    assert_eq!(
        prepared.binding_file,
        Some(fixture.bindings.canonicalize().unwrap())
    );
    assert_eq!(prepared.bindings.inputs().len(), 1);
    assert_eq!(prepared.material_paths.len(), 1);
    assert_eq!(
        prepared.project_file,
        fixture.project.canonicalize().unwrap()
    );
}

#[test]
fn planning_rejects_bindings_for_another_plan() {
    let fixture = BindingFixture::new();
    let mut manifest: veac_artifact::ExecutionBindingManifest =
        serde_json::from_slice(&std::fs::read(&fixture.bindings).unwrap()).unwrap();
    manifest.plan_hash = "00".repeat(32);
    std::fs::write(
        &fixture.bindings,
        veac_artifact::canonical_binding_bytes(&manifest).unwrap(),
    )
    .unwrap();

    let error = crate::planning::prepare_with_bindings(
        &fixture.project,
        None,
        Some(&fixture.bindings),
        &fixture.environment,
    )
    .unwrap_err();
    assert!(error.to_string().contains("EXECUTION_BINDINGS_FAILED"));
}

struct BindingFixture {
    _temp: tempfile::TempDir,
    project: std::path::PathBuf,
    bindings: std::path::PathBuf,
    environment: FakeEnvironment,
}

impl BindingFixture {
    fn new() -> Self {
        let temp = tempdir().unwrap();
        let media = temp.path().join("clip.mp4");
        std::fs::write(&media, b"binding fixture").unwrap();
        let project = canonical_project(&temp, MEDIA_SOURCE);
        let mut environment = FakeEnvironment::success();
        environment.observed = veac_runtime::asset::sha256_identity(&media).unwrap();
        let prepared = crate::planning::prepare(&project, None, &environment).unwrap();
        std::fs::write(
            &project,
            veac_ir::canonical_json(&prepared.project).unwrap(),
        )
        .unwrap();
        let manifest = veac_artifact::binding_manifest(&prepared.plan, &prepared.bindings).unwrap();
        let bindings = temp.path().join("bindings.json");
        std::fs::write(
            &bindings,
            veac_artifact::canonical_binding_bytes(&manifest).unwrap(),
        )
        .unwrap();
        Self {
            _temp: temp,
            project,
            bindings,
            environment,
        }
    }
}
