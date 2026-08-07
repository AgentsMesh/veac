use tempfile::tempdir;
use veac_artifact::ArtifactStore;

use super::substitution_command_tests::segment_contract;
use super::support::{canonical_project, FakeEnvironment, GENERATED_SOURCE};
use crate::arguments::SubstitutionPolicy::{Original, Prefer, Require};

#[test]
fn semantic_segment_poison_is_rejected_before_binding_or_promotion() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let environment = FakeEnvironment::success();
    let prepared =
        crate::planning::prepare_with_material_root(&project, None, None, &environment).unwrap();
    let contract = segment_contract(&prepared, &environment);
    ArtifactStore::new(temp.path().join(".veac-artifacts"))
        .put(contract.descriptor(), b"semantic-invalid")
        .unwrap();

    crate::commands::render(
        &project,
        None,
        crate::planning::InputResolution::default(),
        None,
        Original,
        Prefer,
        &environment,
    )
    .unwrap();
    let calls = environment.executed.borrow();
    assert_eq!(calls.len(), 1);
    assert!(!calls[0].windows(2).any(|pair| pair == ["-c", "copy"]));
    drop(calls);

    let error = crate::commands::render(
        &project,
        None,
        crate::planning::InputResolution::default(),
        None,
        Original,
        Require,
        &environment,
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("RENDER_SEGMENT_POSTFLIGHT_FAILED"));
    assert_eq!(environment.executed.borrow().len(), 1);
}
