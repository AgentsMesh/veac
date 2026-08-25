use super::super::parse_executable;

#[test]
fn constant_span_includes_semicolon_and_expression_span_is_trimmed() {
    let source = "const time base =   1s + 2s   ;";
    let file = parse_executable("main.veac", source).unwrap();
    let value = &file.constants[0];

    assert_eq!(&source[value.span.start..value.span.end], source);
    assert_eq!(
        &source[value.expression_span.start..value.expression_span.end],
        "1s + 2s"
    );
    assert_eq!(file.syntax.slice_text(&value.expression), "1s + 2s");
}

#[test]
fn exported_constant_owns_const_through_semicolon_but_not_export() {
    let source = "module {\n  export const time public = 3s ;\n}";
    let file = parse_executable("timing.veac", source).unwrap();
    let value = &file.constants[0];

    assert!(value.exported);
    assert_eq!(
        &source[value.span.start..value.span.end],
        "const time public = 3s ;"
    );
    assert_eq!(
        &source[value.expression_span.start..value.expression_span.end],
        "3s"
    );
}
