use std::sync::Arc;

use super::super::{
    compile_expression, evaluate_in, ExpressionContext, PrimitiveType, TypeEnvironment, ValueType,
};
use crate::program::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, StructDefinition,
    TypeDefinition, TypeDefinitionKind, TypeRegistry, TypeRegistryBuilder, VariantIndex,
};

fn definition(name: &str, kind: TypeDefinitionKind) -> Arc<TypeDefinition> {
    Arc::new(TypeDefinition::new("brand.veac", name, kind))
}

fn card() -> Arc<TypeDefinition> {
    definition(
        "Card",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "title",
            ValueType::primitive(PrimitiveType::Text),
        )])),
    )
}

fn mood() -> Arc<TypeDefinition> {
    definition(
        "Mood",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![EnumVariantDefinition::new(
            VariantIndex::new(0),
            "Calm",
            vec![],
        )])),
    )
}

fn registry() -> TypeRegistry {
    let card = card();
    let mood = mood();
    let mut builder = TypeRegistryBuilder::new();
    for (name, value) in [("Card", &card), ("Mood", &mood)] {
        builder.insert(Arc::clone(value)).unwrap();
        builder.bind(name, value.type_ref().clone()).unwrap();
        builder
            .bind(
                format!("brand.{name}"),
                value
                    .type_ref()
                    .with_diagnostic_name(format!("brand.{name}")),
            )
            .unwrap();
    }
    builder.finish().unwrap()
}

fn context() -> ExpressionContext {
    ExpressionContext::empty().with_types(Arc::new(registry()))
}

const CAPTURE: &str = r#"{
  let captured: brand.Card = brand.Card { title: "captured" };
  let callback = fn(input: brand.Card) -> brand.Card effect pure { captured };
  callback(brand.Card { title: "input" })
}"#;

#[test]
fn nominal_closure_types_reach_verified_core_by_stable_identity() {
    let context = context();
    let compiled = compile_expression(CAPTURE, &TypeEnvironment::new(), &context).unwrap();
    let closure = &compiled.core().closure_definitions()[0];
    assert_eq!(closure.parameter_types()[0].to_string(), "brand.Card");
    assert_eq!(closure.capture_types()[0].to_string(), "brand.Card");
    assert_eq!(compiled.core().nominal_definitions().len(), 1);
}

#[test]
fn nominal_closure_capture_and_invoke_execute() {
    let value = evaluate_in(CAPTURE, &Default::default(), &context()).unwrap();
    assert_eq!(value.render(), "Card { title: \"captured\" }");
    let mood = evaluate_in(
        "(fn(value: brand.Mood) -> brand.Mood effect pure { value })(brand.Mood.Calm)",
        &Default::default(),
        &context(),
    )
    .unwrap();
    assert_eq!(mood.render(), "Mood.Calm");
}

#[test]
fn nominal_annotation_resolution_fails_closed() {
    let source = "fn(value: Missing) -> Missing effect pure { value }";
    let error = compile_expression(source, &TypeEnvironment::new(), &context()).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_TYPE_SYNTAX");
    assert!(error.message().contains("PROGRAM_UNKNOWN_TYPE"));
    assert_eq!(&source[error.span()], "Missing");

    let hidden = definition(
        "Hidden",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![])),
    );
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(hidden).unwrap();
    let private = ExpressionContext::empty().with_types(Arc::new(builder.finish().unwrap()));
    let source = "fn(value: brand.Hidden) -> brand.Hidden effect pure { value }";
    let error = compile_expression(source, &TypeEnvironment::new(), &private).unwrap_err();
    assert!(error.message().contains("brand.Hidden"));
}

#[test]
fn ambiguous_type_aliases_are_rejected_before_annotation_resolution() {
    let card = card();
    let mood = mood();
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::clone(&card)).unwrap();
    builder.insert(Arc::clone(&mood)).unwrap();
    builder
        .bind("shared.Value", card.type_ref().clone())
        .unwrap();
    let error = builder
        .bind("shared.Value", mood.type_ref().clone())
        .unwrap_err();
    assert_eq!(error.code(), "TYPE_DUPLICATE_NAME");
}
