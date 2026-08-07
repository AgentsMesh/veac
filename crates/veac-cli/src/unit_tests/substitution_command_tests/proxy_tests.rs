use tempfile::tempdir;
use veac_artifact::ArtifactStore;

use super::super::support::{canonical_project, pin_first_material, FakeEnvironment, MEDIA_SOURCE};
use super::proxy_descriptor;
use crate::arguments::SubstitutionPolicy::{Original, Prefer, Require};

#[test]
fn required_exact_proxy_becomes_the_physical_backend_input() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), b"original").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut environment = FakeEnvironment::success();
    environment.observed =
        veac_runtime::asset::sha256_identity(&temp.path().join("clip.mp4")).unwrap();
    pin_first_material(&project, environment.observed.clone());
    let prepared =
        crate::planning::prepare_with_material_root(&project, None, None, &environment).unwrap();
    let descriptor = proxy_descriptor(&prepared, &environment);
    let store = ArtifactStore::new(temp.path().join(".veac-artifacts"));
    let record = store.put(&descriptor, b"exact proxy").unwrap();
    let payload = store
        .open(&record.key)
        .unwrap()
        .unwrap()
        .payload_path()
        .to_owned();

    crate::commands::render(
        &project,
        None,
        crate::planning::InputResolution::default(),
        None,
        Require,
        Original,
        &environment,
    )
    .unwrap();
    let calls = environment.executed.borrow();
    assert_eq!(calls.len(), 1);
    assert_ne!(input(&calls[0]), payload.to_str().unwrap());
    assert!(!calls[0].iter().any(|value| value.ends_with("clip.mp4")));
    assert_eq!(
        environment.consumed_inputs.borrow()[0],
        vec![b"exact proxy".to_vec()]
    );
}

#[test]
fn required_proxy_miss_fails_while_prefer_safely_falls_back_from_corruption() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("clip.mp4"), b"original").unwrap();
    let project = canonical_project(&temp, MEDIA_SOURCE);
    let mut environment = FakeEnvironment::success();
    environment.observed =
        veac_runtime::asset::sha256_identity(&temp.path().join("clip.mp4")).unwrap();
    pin_first_material(&project, environment.observed.clone());
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
    assert!(error.to_string().contains("PROXY_REQUIRED_MISSING"));
    assert!(environment.executed.borrow().is_empty());

    let prepared =
        crate::planning::prepare_with_material_root(&project, None, None, &environment).unwrap();
    let descriptor = proxy_descriptor(&prepared, &environment);
    let store = ArtifactStore::new(temp.path().join(".veac-artifacts"));
    let record = store.put(&descriptor, b"exact proxy").unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    std::fs::write(artifact.payload_path(), b"corrupt").unwrap();
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

fn input(arguments: &[String]) -> &str {
    arguments
        .windows(2)
        .find(|pair| pair[0] == "-i")
        .map(|pair| pair[1].as_str())
        .unwrap()
}
