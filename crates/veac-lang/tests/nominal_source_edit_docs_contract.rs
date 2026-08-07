use std::{fs, path::PathBuf};

use veac_lang::program::SOURCE_INDEX_SCHEMA_VERSION;
use veac_lang::source_edit::SOURCE_EDIT_SCHEMA_VERSION;

#[test]
fn source_edit_references_track_nominal_declaration_contracts() {
    assert_eq!(SOURCE_INDEX_SCHEMA_VERSION, 8);
    assert_eq!(SOURCE_EDIT_SCHEMA_VERSION, 6);
    let editing = read("docs/language-reference/source-editing.md");
    let addressing = read("docs/language-reference/source-addressing.md");
    let nominal = read("docs/language-reference/programming-nominal.md");
    for token in [
        "source-index v8",
        "source-edit v6",
        "`DeclarationSite`",
        "`declaration_equals`/`set_declaration`",
    ] {
        assert!(nominal.contains(token), "nominal reference omitted {token}");
    }
    assert!(editing.contains("`source-index` v8"));
    assert!(editing.contains("declaration_equals`/`set_declaration"));
    assert!(editing.contains("statement_equals`/`set_statement"));
    for path in [
        "struct_field:       structure + field",
        "enum_variant:       enumeration + variant",
        "enum_variant_field: enumeration + variant + field",
    ] {
        assert!(addressing.contains(path), "addressing omitted {path}");
    }
}

#[test]
fn generated_schemas_expose_closed_nominal_declaration_variants() {
    let source_edit = veac_lang::source_edit::source_edit_batch_json_schema()
        .unwrap()
        .to_string();
    let source_index = veac_lang::program::source_index_json_schema()
        .unwrap()
        .to_string();
    for token in [
        "set_declaration",
        "declaration_equals",
        "set_top_level_declaration",
        "insert_declaration",
        "insert_import",
    ] {
        assert!(
            source_edit.contains(token),
            "source-edit schema omitted {token}"
        );
    }
    for token in [
        "modules",
        "imports",
        "declarations",
        "implementation",
        "temporal",
        "enum_variant_field_declaration",
        "build_inputs",
    ] {
        assert!(
            source_index.contains(token),
            "source-index schema omitted {token}"
        );
    }
}

fn read(relative: &str) -> String {
    fs::read_to_string(workspace_root().join(relative)).unwrap()
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
