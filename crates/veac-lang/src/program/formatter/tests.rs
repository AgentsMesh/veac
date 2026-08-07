use super::*;

#[path = "tests/comments.rs"]
mod comments;
#[path = "tests/contracts.rs"]
mod contracts;

pub(super) const ENTRY: &str = r#"fn main(context: Context) -> Project {
  let timeline = sequence(
    identifier("main"), "格式化测试",
    sequence_settings(canvas(64px, 36px), frame_rate(30, 1), 48000)
  );
  project(identifier("formatter"), project_settings(600))
    .with_sequence(timeline)
    .entry(timeline)
}
"#;

#[test]
fn entry_formatting_is_idempotent_and_has_one_final_newline() {
    let messy = ENTRY.replace(" {\n", "{\n\n").replace("  let", "\tlet");
    let once = format_source(&messy).unwrap_or_else(|error| {
        let tokens = lexer::lex("main.veac", &messy).unwrap();
        panic!("{error:?}\n{}", layout::format(&messy, &tokens));
    });
    let twice = format_source(&once).unwrap();
    assert_eq!(once, twice);
    assert!(once.ends_with('\n'));
    assert!(!once.ends_with("\n\n"));
    assert_ne!(once, messy);
}

#[test]
fn trivia_scanner_preserves_comment_text() {
    let parsed = trivia::parse("  /* one\n */ \n // two\r\n  ");
    assert_eq!(parsed.comments.len(), 2);
    assert_eq!(parsed.comments[0].leading, "  ");
    assert_eq!(parsed.comments[0].text, "/* one\n */");
    assert!(!parsed.comments[0].line);
    assert_eq!(parsed.comments[1].text, "// two\r");
    assert!(parsed.comments[1].line);
    assert_eq!(parsed.tail, "\n  ");
}

#[test]
fn path_formatting_maps_loader_failures_to_a_source_diagnostic() {
    let missing = std::path::Path::new("missing-formatter-entry.veac");
    let diagnostics = format_path(missing).unwrap_err();
    let error = &diagnostics.as_slice()[0];
    assert_eq!(error.code, "PROGRAM_FORMAT_LOAD");
    assert_eq!(error.path, missing.display().to_string());
}
