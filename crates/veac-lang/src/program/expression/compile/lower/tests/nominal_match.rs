use std::sync::Arc;

use super::super::*;
use crate::program::expression::{ExpressionContext, PrimitiveType, ValueType};
use crate::program::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, StructDefinition,
    TypeDefinition, TypeDefinitionKind, TypeRegistryBuilder, VariantIndex,
};

fn field(index: u16, name: &str, value_type: ValueType) -> FieldDefinition {
    FieldDefinition::new(FieldIndex::new(index), name, value_type)
}

fn definition(name: &str, kind: TypeDefinitionKind) -> Arc<TypeDefinition> {
    Arc::new(TypeDefinition::new("lower-tests.veac", name, kind))
}

fn context() -> ExpressionContext {
    let card = definition(
        "Card",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![
            field(0, "title", PrimitiveType::Text.into()),
            field(1, "duration", PrimitiveType::Time.into()),
        ])),
    );
    let choice = definition(
        "Choice",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![
            EnumVariantDefinition::new(VariantIndex::new(0), "Empty", vec![]),
            EnumVariantDefinition::new(
                VariantIndex::new(1),
                "Pair",
                vec![
                    field(0, "first", PrimitiveType::Time.into()),
                    field(1, "second", PrimitiveType::Time.into()),
                ],
            ),
            EnumVariantDefinition::new(VariantIndex::new(2), "Other", vec![]),
        ])),
    );
    let alternate = definition(
        "Alternate",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![EnumVariantDefinition::new(
            VariantIndex::new(0),
            "Only",
            vec![],
        )])),
    );
    let mut builder = TypeRegistryBuilder::new();
    for value in [card, choice, alternate] {
        builder.insert(Arc::clone(&value)).unwrap();
        builder
            .bind(value.declared_name(), value.type_ref().clone())
            .unwrap();
    }
    ExpressionContext::empty().with_types(Arc::new(builder.finish().unwrap()))
}

fn lowered(source: &str) -> Result<TypedExpression, ExpressionError> {
    let tokens = crate::program::expression::lexer::lex(source)?;
    let parsed = crate::program::expression::parser::parse(tokens)?;
    expression(&parsed, &|_| None, &context())
}

fn error(source: &str) -> &'static str {
    lowered(source).unwrap_err().code()
}

#[test]
fn nominal_construction_and_match_lower_all_layout_forms() {
    for source in [
        r#"Card { duration: 2s, title: "card" }"#,
        "Choice.Empty",
        "Choice.Pair { second: 2s, first: 1s }",
        "match (Choice.Pair { first: 1s, second: 2s }) {\
           Choice.Empty => 0s,\
           Choice.Pair { first, second: tail } => first + tail,\
           Choice.Other => 3s, }",
        "match Choice.Empty { Choice.Pair { first, second } => first + second, _ => 0s, }",
    ] {
        lowered(source).unwrap();
    }
}

#[test]
fn match_infers_collection_context_from_later_or_expected_arms() {
    let inferred = lowered(
        "match Choice.Empty { Choice.Empty => [], Choice.Pair { first, second } => [1], \
         Choice.Other => [2], }",
    )
    .unwrap();
    assert_eq!(inferred.result_type().to_string(), "list<int>");
    let expected = lowered(
        "{ let values: list<int> = match Choice.Empty { Choice.Empty => [], _ => [], }; values }",
    )
    .unwrap();
    assert_eq!(expected.result_type().to_string(), "list<int>");
    assert_eq!(
        error(
            "match Choice.Empty { Choice.Empty => [], Choice.Pair { first, second } => [], \
             Choice.Other => [], }",
        ),
        "EXPRESSION_COLLECTION_TYPE_CONTEXT"
    );
    assert_eq!(
        error(
            "match Choice.Empty { Choice.Empty => [], Choice.Pair { first, second } => missing, \
             Choice.Other => [1], }",
        ),
        "EXPRESSION_UNKNOWN_SYMBOL"
    );
}

#[test]
fn nominal_construction_reports_kind_variant_and_field_errors() {
    let cases = [
        ("Missing {}", "EXPRESSION_UNKNOWN_TYPE"),
        (
            r#"Card.Extra { title: "x", duration: 1s }"#,
            "EXPRESSION_NOMINAL_KIND",
        ),
        ("Choice {}", "EXPRESSION_NOMINAL_KIND"),
        ("Choice.Missing {}", "EXPRESSION_UNKNOWN_VARIANT"),
        ("Choice.Missing", "EXPRESSION_UNKNOWN_VARIANT"),
        ("Choice.Pair", "EXPRESSION_ENUM_PAYLOAD"),
        (r#"Card { missing: "x" }"#, "EXPRESSION_UNKNOWN_FIELD"),
        (r#"Card { title: "x" }"#, "EXPRESSION_MISSING_NOMINAL_FIELD"),
        (
            r#"Card { title: "x", duration: 1px }"#,
            "EXPRESSION_NOMINAL_FIELD_TYPE",
        ),
        (
            r#"{ let Card = 1; Card { title: "x", duration: 1s } }"#,
            "EXPRESSION_NOMINAL_SHADOWED",
        ),
    ];
    for (source, code) in cases {
        assert_eq!(error(source), code, "{source}");
    }
}

#[test]
fn match_resolution_rejects_invalid_coverage_and_patterns() {
    let pair = "(Choice.Pair { first: 1s, second: 2s })";
    let cases = [
        ("match 1 { _ => 2, }".to_owned(), "EXPRESSION_MATCH_TYPE"),
        (
            r#"match (Card { title: "x", duration: 1s }) { _ => 1, }"#.to_owned(),
            "EXPRESSION_MATCH_TYPE",
        ),
        (
            format!("match {pair} {{ Choice.Pair {{ first, second }} => first, }}"),
            "EXPRESSION_NON_EXHAUSTIVE_MATCH",
        ),
        (
            format!("match {pair} {{ Choice.Pair {{ first, second }} => first, Choice.Pair {{ first: a, second: b }} => a, _ => 0s, }}"),
            "EXPRESSION_DUPLICATE_MATCH_VARIANT",
        ),
        (
            format!("match {pair} {{ Choice.Empty => 0s, Choice.Pair {{ first, second }} => first, Choice.Other => 0s, _ => 1s, }}"),
            "EXPRESSION_UNREACHABLE_WILDCARD",
        ),
        (
            format!("match {pair} {{ Missing.Pair {{ first, second }} => first, _ => 0s, }}"),
            "EXPRESSION_UNKNOWN_TYPE",
        ),
        (
            format!("match {pair} {{ Alternate.Only => 0s, _ => 1s, }}"),
            "EXPRESSION_MATCH_PATTERN_TYPE",
        ),
        (
            format!("match {pair} {{ Choice.Pair.More => 0s, _ => 1s, }}"),
            "EXPRESSION_MATCH_PATTERN",
        ),
        (
            format!("match {pair} {{ Choice.Missing => 0s, _ => 1s, }}"),
            "EXPRESSION_UNKNOWN_VARIANT",
        ),
        (
            format!("match {pair} {{ Choice.Pair {{ missing }} => 0s, _ => 1s, }}"),
            "EXPRESSION_UNKNOWN_PATTERN_FIELD",
        ),
        (
            format!("match {pair} {{ Choice.Pair {{ first }} => first, _ => 0s, }}"),
            "EXPRESSION_INCOMPLETE_PATTERN",
        ),
        (
            format!("match {pair} {{ Choice.Empty => 0s, Choice.Pair {{ first, second }} => 1, Choice.Other => 2s, }}"),
            "EXPRESSION_MATCH_ARM_TYPE",
        ),
    ];
    for (source, code) in cases {
        assert_eq!(error(&source), code, "{source}");
    }
}
