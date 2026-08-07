use super::*;

#[test]
fn graph_revision_is_order_independent_and_byte_exact() {
    let first = SourceModule::utf8(
        "main.veac",
        "fn main(context: Context) -> Project { context }",
    );
    let second = SourceModule::utf8("parts/title.veac", "module { export const title = \"x\"; }");
    let forward = source_graph_revision(&[first, second]).unwrap();
    let reverse = source_graph_revision(&[second, first]).unwrap();
    assert_eq!(forward, reverse);
    assert_eq!(forward.source_graph_sha256.len(), 64);

    let changed = SourceModule::utf8(
        "main.veac",
        "fn main(context: Context) -> Project { context }\n",
    );
    assert_ne!(source_graph_revision(&[changed, second]).unwrap(), forward);
}

#[test]
fn graph_hash_framing_separates_paths_and_content() {
    let left =
        source_graph_revision(&[SourceModule::utf8("a", "bc"), SourceModule::utf8("def", "")])
            .unwrap();
    let right =
        source_graph_revision(&[SourceModule::utf8("ab", "c"), SourceModule::utf8("def", "")])
            .unwrap();
    assert_ne!(left, right);
}

#[test]
fn graph_revision_rejects_empty_duplicate_and_unsafe_paths() {
    assert_eq!(
        source_graph_revision(&[]),
        Err(SourceEditError::EmptySourceGraph)
    );
    let duplicate = SourceModule::utf8("main.veac", "");
    assert!(matches!(
        source_graph_revision(&[duplicate, duplicate]),
        Err(SourceEditError::DuplicateModulePath(_))
    ));
    for path in [
        "",
        "/tmp/main.veac",
        "../main.veac",
        "a//b",
        "a\\b",
        "C:/main.veac",
        "a/\u{7f}b",
        "a/\u{85}b",
        "a/\0b",
        ".veac-source.lock",
        "nested/.veac-source.lock",
        &"a".repeat(4097),
    ] {
        assert!(matches!(
            validate_module_path(path),
            Err(SourceEditError::InvalidModulePath(_))
        ));
    }
}

#[test]
fn graph_revision_accepts_normalized_relative_nested_paths() {
    assert!(validate_module_path("parts/片头.veac").is_ok());
    assert!(source_graph_revision(&[SourceModule::new("main.veac", &[0xff])]).is_ok());
}

#[test]
fn sha256_validation_requires_canonical_lowercase_hex() {
    assert!(valid_sha256(&"0".repeat(64)));
    assert!(!valid_sha256(&"A".repeat(64)));
    assert!(!valid_sha256(&"0".repeat(63)));
}
