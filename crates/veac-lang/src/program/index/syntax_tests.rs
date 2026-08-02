use crate::authoring::Span;

use super::syntax;

#[test]
fn parses_nested_blocks_placeholders_and_exact_expression_spans() {
    let source = "outer { value ${base + 1s}; text \"brace } stays text\"; }";
    let root = parse(source).unwrap();
    let outer = &root.entries[0].block.as_ref().unwrap().entries;
    let (value, span) = outer[0].expression(source, 1).unwrap();
    assert_eq!(value, "${base + 1s}");
    assert_eq!(&source[span.start..span.end], value);
    assert_eq!(
        outer[1].expression(source, 1).unwrap().0,
        "\"brace } stays text\""
    );
}

#[test]
fn rejects_each_unbalanced_or_unterminated_structure() {
    for (source, expected) in [
        ("}", "unexpected closing brace"),
        ("field", "source entry is not terminated"),
        ("field }", "source entry requires `;` or a block"),
        ("outer { field;", "nested source block is not closed"),
        ("field ${value", "expression placeholder is not closed"),
        (";", "source entry has no name"),
    ] {
        let error = parse(source).unwrap_err();
        assert_eq!(error.code, "SOURCE_INDEX_SYNTAX");
        assert_eq!(error.message, expected);
    }
}

#[test]
fn forwards_program_lexer_errors() {
    let error = parse("invalid ?;").unwrap_err();
    assert_eq!(error.code, "PROGRAM_LEX_TOKEN");
}

#[test]
fn expression_fragments_accept_authored_units_but_reject_structure() {
    for source in ["1stops", "-2db", "pcm-s24le", "\"safe; value\""] {
        assert!(syntax::validate_expression_fragment(source).is_ok());
    }
    for source in ["", "1; other 2", "value { nested 1; }", "1 }"] {
        assert!(syntax::validate_expression_fragment(source).is_err());
    }
}

fn parse(source: &str) -> Result<syntax::Block, crate::program::Diagnostic> {
    syntax::parse(
        "main.veac",
        source,
        Span {
            start: 0,
            end: source.len(),
        },
    )
}
