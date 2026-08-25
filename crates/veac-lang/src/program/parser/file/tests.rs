use super::parse;

const SOURCE: &str = r#"/* exact source */
const time base = 1s /* constant */;
fn scale(value: scalar) -> scalar { value /* function */ }
struct Item { value: scalar }
impl Item @local {
  fn read(self) -> scalar { self.value /* method */ }
}
animate visual-opacity on clip(@demo, @main, @visual, @first) {
  progress /* animation */
}
"#;

#[test]
fn production_declarations_own_absolute_slices_of_one_document() {
    let file = parse("slices.veac", SOURCE).unwrap();
    assert_eq!(file.source().as_bytes(), SOURCE.as_bytes());

    let constant = &file.constants[0];
    assert_eq!(file.syntax.slice_text(&constant.expression), "1s");
    assert_eq!(token_text(&file, &constant.expression), ["1s"]);

    let function = &file.functions[0];
    assert_eq!(
        file.syntax.slice_text(&function.body.syntax),
        "{ value /* function */ }"
    );
    assert_eq!(
        file.syntax.slice_text(&function.syntax),
        &SOURCE[function.span.start..function.span.end]
    );

    let implementation = &file.implementations[0];
    let method = &implementation.methods[0];
    assert_eq!(
        file.syntax.slice_text(&method.body.syntax),
        "{ self.value /* method */ }"
    );
    assert_eq!(
        file.syntax.slice_text(&implementation.syntax),
        &SOURCE[implementation.span.start..implementation.span.end]
    );

    let temporal = &file.temporal[0];
    assert!(file
        .syntax
        .slice_text(&temporal.body.syntax)
        .contains("progress"));
    assert_eq!(
        file.syntax.slice_text(&temporal.syntax),
        &SOURCE[temporal.span.start..temporal.span.end]
    );
}

fn token_text<'a>(
    file: &'a crate::program::model::SurfaceFile,
    slice: &crate::program::syntax_document::SyntaxSlice,
) -> Vec<&'a str> {
    file.syntax
        .tokens()
        .get(slice.tokens.clone())
        .expect("declaration slice must address syntax tokens")
        .iter()
        .map(|token| file.syntax.text(token.span))
        .collect()
}

#[test]
fn expression_error_span_stays_absolute_after_leading_utf8_and_comments() {
    let source = "// \u{6807}\u{9898}\nconst time value = ;";
    let error = parse("span.veac", source).unwrap_err().remove(0);
    let semicolon = source.find(';').unwrap();
    assert_eq!(error.code, "PROGRAM_EMPTY_EXPRESSION");
    assert_eq!((error.span.start, error.span.end), (semicolon, semicolon));
}
