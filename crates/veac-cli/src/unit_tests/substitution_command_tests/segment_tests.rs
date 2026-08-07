use tempfile::tempdir;
use veac_artifact::{select_full_render_segment, ArtifactStore};

use super::super::support::{canonical_project, FakeEnvironment, GENERATED_SOURCE};
use super::segment_contract;
use crate::arguments::SubstitutionPolicy::{Original, Prefer, Require};

#[test]
fn preferred_segment_is_stored_then_reused_and_corruption_never_falls_back() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let environment = FakeEnvironment::success();
    let prepared =
        crate::planning::prepare_with_material_root(&project, None, None, &environment).unwrap();
    let contract = segment_contract(&prepared, &environment);
    let store = ArtifactStore::new(temp.path().join(".veac-artifacts"));

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
    let segment = select_full_render_segment(&store, &contract)
        .unwrap()
        .expect("first render stores its exact full segment");
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
    assert_eq!(calls.len(), 2);
    assert!(calls[1].windows(2).any(|pair| pair == ["-c", "copy"]));
    assert_ne!(input(&calls[1]), segment.payload_path().to_str().unwrap());
    assert_eq!(
        environment.consumed_inputs.borrow()[1],
        vec![b"rendered".to_vec()]
    );
    drop(calls);

    std::fs::write(segment.payload_path(), b"corrupt").unwrap();
    let error = crate::commands::render(
        &project,
        None,
        crate::planning::InputResolution::default(),
        None,
        Original,
        Prefer,
        &environment,
    )
    .unwrap_err();
    assert!(error.to_string().contains("RENDER_SEGMENT_FAILED"));
    assert_eq!(environment.executed.borrow().len(), 2);
}

#[test]
fn required_segment_miss_starts_no_ffmpeg_task() {
    let temp = tempdir().unwrap();
    let project = canonical_project(&temp, GENERATED_SOURCE);
    let environment = FakeEnvironment::success();
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
        .contains("RENDER_SEGMENT_REQUIRED_MISSING"));
    assert!(environment.executed.borrow().is_empty());
}

fn input(arguments: &[String]) -> &str {
    arguments
        .windows(2)
        .find(|pair| pair[0] == "-i")
        .map(|pair| pair[1].as_str())
        .unwrap()
}
