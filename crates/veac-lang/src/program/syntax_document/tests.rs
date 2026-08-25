use super::{SyntaxElementKind, TriviaKind};
use crate::program::lexer;

#[test]
fn document_roundtrips_every_byte_and_preserves_trivia_ownership() {
    let source = "\u{03bb}  // keep\n/* block */fn main() -> scalar { 1 }\n";
    let document = lexer::lex_document("roundtrip.veac", source).unwrap();
    assert_eq!(document.source(), source);
    assert_eq!(document.source().as_bytes(), source.as_bytes());
    assert!(document
        .elements
        .iter()
        .any(|item| item.kind == SyntaxElementKind::Trivia(TriviaKind::LineComment)));
    assert!(document
        .elements
        .iter()
        .any(|item| item.kind == SyntaxElementKind::Trivia(TriviaKind::BlockComment)));
    let mut cursor = 0;
    for element in document.elements.iter() {
        assert_eq!(element.span.start, cursor);
        cursor = element.span.end;
    }
    assert_eq!(cursor, source.len());
}

#[test]
fn empty_and_comment_only_documents_have_a_valid_lossless_shape() {
    for source in ["", " // only a comment\n", "/* block */"] {
        let document = lexer::lex_document("empty.veac", source).unwrap();
        assert_eq!(document.source(), source);
        assert_eq!(
            document.elements.last().map(|value| value.span.end),
            Some(source.len()).filter(|_| !source.is_empty())
        );
    }
}
