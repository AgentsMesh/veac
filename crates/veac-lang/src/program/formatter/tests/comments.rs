use super::super::*;

#[test]
fn line_and_block_comments_keep_text_order_and_ownership() {
    let source = concat!(
        "module { // attached to opener   \n",
        "  /* standalone\n",
        "     block */\n",
        "  export const text label = \"// string, not a comment\"; /* trailing block */\n",
        "  // before duration\n",
        "  export const time duration = 200ms;\n",
        "}\n",
        "// final comment   ",
    );
    let formatted = format_source(source).unwrap();
    assert_eq!(comments(source), comments(&formatted));
    assert!(formatted.contains("module { // attached to opener   \n"));
    assert!(formatted.contains("200ms;\n"));
    assert!(formatted.ends_with("// final comment   \n"));
    assert_eq!(format_source(&formatted).unwrap(), formatted);
}

#[test]
fn token_spellings_inside_literals_are_untouched() {
    let source = r#"module {
 export fn label() -> text { "  #Aa00 // exact /* value */  " }
 export fn accent() -> color { #112233ff }
 export fn ratio() -> scalar { 1.00 }
}
"#;
    let formatted = format_source(source).unwrap();
    for exact in ["\"  #Aa00 // exact /* value */  \"", "#112233ff", "1.00"] {
        assert!(formatted.contains(exact), "missing {exact} in {formatted}");
    }
}

#[test]
fn leading_comments_keep_inline_or_standalone_attachment() {
    let inline = "/* inline header */ module { export const time value = 1s; }";
    assert!(format_source(inline)
        .unwrap()
        .starts_with("/* inline header */ module {"));

    let standalone = "\n\n// file header\nmodule { export const time value = 1s; }";
    let formatted = format_source(standalone).unwrap();
    assert!(formatted.starts_with("// file header\nmodule {"));
    assert_eq!(format_source(&formatted).unwrap(), formatted);
}

#[test]
fn preservation_guard_detects_comment_text_or_gap_changes() {
    let left = "module { /* left */ export const time value = 1s; }";
    let changed = "module { /* changed */ export const time value = 1s; }";
    let moved = "/* left */ module { export const time value = 1s; }";
    let tokens = |source| lexer::lex("comments.veac", source).unwrap();
    assert!(!preservation::same_comments(
        left,
        &tokens(left),
        changed,
        &tokens(changed),
    ));
    assert!(!preservation::same_comments(
        left,
        &tokens(left),
        moved,
        &tokens(moved),
    ));
}

fn comments(source: &str) -> Vec<String> {
    let tokens = lexer::lex("comments.veac", source).unwrap();
    let mut offset = 0;
    let mut result = Vec::new();
    for token in tokens {
        result.extend(
            trivia::parse(&source[offset..token.span.start])
                .comments
                .into_iter()
                .map(|comment| comment.text.to_owned()),
        );
        offset = token.span.end;
    }
    result
}
