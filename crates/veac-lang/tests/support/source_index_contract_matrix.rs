use crate::program::prepare_source;
use crate::source_edit::{
    BodySite, DeclarationSite, ExpressionSite, SourceNodeRef, SourceSnapshot,
};

const SOURCE: &str = r#"
const int answer = 42;
struct Label { value: text, }
fn helper(value: int) -> int { value }
fn main(context: Context) -> Project {
  let timeline = sequence(identifier("main"), "索引契约",
    sequence_settings(canvas(16px, 16px), frame_rate(1, 1), 8000));
  project(identifier("index"), project_settings(60))
    .with_sequence(timeline).entry(timeline)
}"#;

#[test]
fn source_index_exposes_expression_body_declaration_and_snapshot_views() {
    let index = prepare_source(SOURCE).unwrap().source_index().unwrap();
    let constant = SourceNodeRef::constant("main.veac", "answer");
    let expression = ExpressionSite::ConstantValue;
    assert_eq!(
        index.expression(&constant, &expression).unwrap().source,
        "42"
    );
    assert_eq!(index.expression_source(&constant, &expression), Some("42"));
    assert!(index.node_exists(&constant));

    let function = SourceNodeRef::function("main.veac", "helper");
    assert_eq!(
        index.body_source(&function, BodySite::FunctionBody),
        Some("{ value }")
    );
    let structure = SourceNodeRef::structure("main.veac", "Label");
    assert!(index
        .declaration_source(&structure, DeclarationSite::StructDeclaration)
        .unwrap()
        .starts_with("struct Label"));

    let missing = SourceNodeRef::constant("main.veac", "missing");
    assert!(!index.node_exists(&missing));
    assert_eq!(index.expression(&missing, &expression), None);
    assert_eq!(index.body(&missing, BodySite::FunctionBody), None);
    assert_eq!(
        index.declaration(&missing, DeclarationSite::StructDeclaration),
        None
    );
}
