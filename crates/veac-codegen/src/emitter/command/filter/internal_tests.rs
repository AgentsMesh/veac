use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::*;

const TOKEN: &str = "__VEAC_FILTER_RESOURCE_0000__";

#[test]
fn internal_files_remain_typed_until_staging() {
    let contract = internal(BackendInternalAccess::Produce);
    assert_eq!(contract.render_original().unwrap(), TOKEN);
    assert_eq!(
        contract
            .render_bound(&BTreeMap::new(), &BTreeMap::new())
            .unwrap(),
        TOKEN
    );
    assert_eq!(
        contract
            .render_internal(TOKEN, Path::new("/tmp/stage:x"))
            .unwrap(),
        r"/tmp/stage\:x/motion.trf"
    );
}

#[test]
fn internal_rendering_rejects_missing_or_repeated_tokens() {
    let contract = internal(BackendInternalAccess::Consume);
    for graph in ["plain".to_owned(), format!("{TOKEN};{TOKEN}")] {
        let error = contract
            .render_internal(&graph, Path::new("/tmp/stage"))
            .unwrap_err();
        assert!(error.contains("exactly once"), "{error}");
    }
}

#[test]
fn internal_access_and_path_are_preserved() {
    let contract = internal(BackendInternalAccess::Consume);
    assert!(matches!(
        &contract.bindings()[0],
        BackendFilterBinding::InternalFile { path, access, .. }
            if path == &PathBuf::from("motion.trf")
                && *access == BackendInternalAccess::Consume
    ));
}

fn internal(access: BackendInternalAccess) -> BackendFilterContract {
    BackendFilterContract::new(
        TOKEN.to_owned(),
        vec![BackendFilterBinding::internal_file(
            TOKEN.to_owned(),
            PathBuf::from("motion.trf"),
            access,
            BackendFilterEscape::Quoted,
        )],
    )
    .unwrap()
}
