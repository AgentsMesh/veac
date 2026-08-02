use super::{empty_entry, error_code};
use crate::program::{check_source, compile_source};

#[test]
fn declarations_and_expression_symbols_share_canonical_names() {
    let source = empty_entry(
        r#"const scalar _base = 1;
const scalar title-card = _base + 2;"#,
    );
    let compiled = compile_source(&source).unwrap();
    assert!(compiled.expanded_source().contains("project test"));

    for name in [
        "标题",
        "with.dot",
        "trailing-",
        "two--dash",
        "true",
        "false",
    ] {
        let source = empty_entry(&format!("const scalar {name} = 1;"));
        assert_eq!(error_code(&source), "PROGRAM_IDENTIFIER", "{name}");
    }
}

#[test]
fn module_identity_comes_only_from_its_source_id() {
    assert!(check_source("brand.veac", "module {}").is_ok());
    let error = check_source("brand.veac", "module brand {}").unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_EXPECTED_TOKEN");
}

#[test]
fn value_types_have_exactly_nine_canonical_spellings() {
    for alias in ["number", "string", "boolean", "resource"] {
        let source = empty_entry(&format!("const {alias} value = 1;"));
        assert_eq!(error_code(&source), "PROGRAM_VALUE_TYPE", "{alias}");
    }
    let declarations = r##"const scalar a = 1;
const time b = 1s;
const length c = 1px;
const percent d = 1%;
const angle e = 1deg;
const text f = "x";
const color g = #ffffffff;
const bool h = true;
const identifier i = identifier("asset");"##;
    assert!(compile_source(&empty_entry(declarations)).is_ok());
}
