use tempfile::tempdir;

use super::support::{canonical_project, FakeEnvironment};
use crate::arguments::SubstitutionPolicy::{Original, Require};

const IMAGE_SOURCE: &str = include_str!("../../tests/fixtures/static-image-proxy.veac");

#[test]
fn required_proxy_policy_keeps_static_image_on_its_original_binding() {
    let temp = tempdir().unwrap();
    std::fs::write(temp.path().join("still.png"), b"original still").unwrap();
    let project = canonical_project(&temp, IMAGE_SOURCE);
    let mut environment = FakeEnvironment::success();
    environment.observed =
        veac_runtime::asset::sha256_identity(&temp.path().join("still.png")).unwrap();
    environment.filters.insert("loop".to_owned());

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

    assert_eq!(environment.executed.borrow().len(), 1);
    assert_eq!(
        environment.consumed_inputs.borrow()[0],
        vec![b"original still".to_vec()]
    );
}
