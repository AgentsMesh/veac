use tempfile::tempdir;
use veac_artifact::ArtifactStore;

use super::support::{canonical_project, pin_first_material, FakeEnvironment, MEDIA_SOURCE};
use crate::arguments::SubstitutionPolicy::{Original, Prefer, Require};

#[test]
fn semantic_proxy_failure_is_required_error_and_prefer_fallback() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), b"original").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut environment = FakeEnvironment::success();
    environment.observed =
        veac_runtime::asset::sha256_identity(&temp.path().join("clip.mp4")).unwrap();
    pin_first_material(&project, environment.observed.clone());
    let prepared =
        crate::planning::prepare_with_material_root(&project, None, None, &environment).unwrap();
    let descriptor = super::substitution_command_tests::proxy_descriptor(&prepared, &environment);
    ArtifactStore::new(temp.path().join(".veac-artifacts"))
        .put(&descriptor, b"semantic-invalid")
        .unwrap();

    let error = crate::commands::render(
        &project,
        None,
        crate::planning::InputResolution::default(),
        None,
        Require,
        Original,
        &environment,
    )
    .unwrap_err();
    assert!(error.to_string().contains("PROXY_POSTFLIGHT_FAILED"));
    assert!(environment.executed.borrow().is_empty());

    crate::commands::render(
        &project,
        None,
        crate::planning::InputResolution::default(),
        None,
        Prefer,
        Original,
        &environment,
    )
    .unwrap();
    assert_eq!(environment.executed.borrow().len(), 1);
    assert_eq!(
        environment.consumed_inputs.borrow()[0],
        vec![b"original".to_vec()]
    );
}
