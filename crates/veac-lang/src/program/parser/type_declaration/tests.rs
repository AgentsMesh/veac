use super::*;

#[test]
fn nominal_fragment_validation_reports_lexical_parse_and_trailing_failures() {
    assert!(validate_fragment("\"", TypeDeclarationFragmentKind::Struct).is_err());
    assert!(validate_fragment("struct {", TypeDeclarationFragmentKind::Struct).is_err());
    let trailing = validate_fragment(
        "struct First {} struct Second {}",
        TypeDeclarationFragmentKind::Struct,
    )
    .unwrap_err();
    assert!(trailing.contains("exactly one"));
}
