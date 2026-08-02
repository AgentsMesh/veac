use crate::authoring::parse;

use super::project;

#[test]
fn core_declaration_ids_use_the_canonical_name_contract() {
    for valid in ["_demo", "title-card"] {
        let source =
            project("sequence main {}").replacen("project demo", &format!("project {valid}"), 1);
        assert!(parse(&source).is_ok(), "{valid}");
    }
    for invalid in ["标题", "trailing-", "two--dash", "true", "false"] {
        let source =
            project("sequence main {}").replacen("project demo", &format!("project {invalid}"), 1);
        let errors = parse(&source).unwrap_err();
        assert_eq!(
            errors.as_slice()[0].code,
            "AUTHORING_IDENTIFIER",
            "{invalid}"
        );
    }
}

#[test]
fn references_and_nested_ids_are_validated_at_parse_time() {
    let source = project(
        r#"resource video 标题 { locator local "input.mp4"; }
sequence main {}"#,
    );
    assert_eq!(
        parse(&source).unwrap_err().as_slice()[0].code,
        "AUTHORING_IDENTIFIER"
    );

    let source = project("sequence main { layer visual true {} }");
    assert_eq!(
        parse(&source).unwrap_err().as_slice()[0].code,
        "AUTHORING_IDENTIFIER"
    );
}
