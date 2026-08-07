use std::{fs, path::PathBuf};

#[test]
fn executable_temporal_reference_tracks_the_published_typed_contract() {
    let reference = read("docs/language-reference/executable-temporal.md");
    for contract in [
        "animate visual-opacity on clip",
        "using resource",
        "compile_temporal_expression",
        "TemporalClockOwner::Item",
        "static topology, dynamic leaf values",
        "declared_inputs_sha256",
        "temporal_animation",
        "不是用户 authoring API",
        "ProjectEnvelope::canonical_json",
    ] {
        assert!(reference.contains(contract), "missing contract: {contract}");
    }
    let index = read("docs/language-reference/README.md");
    assert!(index.contains("[可执行 Temporal residualization、typed binding 与 cache identity]"));
    for obsolete in [
        "with_temporal_leaf",
        "push_temporal_leaf",
        "正式的可注入 host boundary",
    ] {
        assert!(
            !reference.contains(obsolete),
            "obsolete host contract: {obsolete}"
        );
    }
}

#[test]
fn executable_docs_no_longer_describe_temporal_as_unpublished() {
    let build = read("docs/language-reference/executable-build.md");
    let rfc = read("docs/rfcs/executable-veac-language.md");
    for obsolete in [
        "当前尚未发布 temporal",
        "temporal residualization 尚未进入发布 envelope",
        "future temporal model",
        "current executable output contains\nno temporal program",
    ] {
        assert!(
            !build.contains(obsolete),
            "obsolete build claim: {obsolete}"
        );
        assert!(!rfc.contains(obsolete), "obsolete RFC claim: {obsolete}");
    }
}

fn read(relative: &str) -> String {
    let path = workspace_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
