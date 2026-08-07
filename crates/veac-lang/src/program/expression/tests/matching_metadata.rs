use std::collections::BTreeMap;
use std::sync::Arc;

use crate::program::expression::{
    compile_expression, CompiledExpression, CoreTerminator, CoreValueMetadata, ExpressionContext,
    InputId, PrimitiveType, TypeEnvironment, ValueType,
};
use crate::program::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, TypeDefinition,
    TypeDefinitionKind, TypeRegistryBuilder, VariantIndex,
};

#[test]
fn single_unit_variant_does_not_taint_constant_arm_with_selector() {
    let (context, choice) = context(&[("Only", false)]);
    let environment = BTreeMap::from([("choice".to_owned(), choice)]);
    let metadata = compile_match(&context, environment, "match choice { Choice.Only => 7, }");

    assert!(metadata.shape_dependencies().is_empty());
    assert!(metadata.leaf_dependencies().is_empty());
}

#[test]
fn single_payload_variant_preserves_the_payload_dependency() {
    let (context, _) = context(&[("Only", true)]);
    let environment = BTreeMap::from([(
        "payload".to_owned(),
        ValueType::primitive(PrimitiveType::Integer),
    )]);
    let metadata = compile_match(
        &context,
        environment,
        "match (Choice.Only { value: payload }) { Choice.Only { value } => value, }",
    );

    assert_eq!(
        metadata.leaf_dependencies().leaf_input_ids(),
        [InputId::new(0)]
    );
}

#[test]
fn multiple_variants_preserve_the_selector_control_dependency() {
    let (context, choice) = context(&[("First", false), ("Second", false)]);
    let environment = BTreeMap::from([("choice".to_owned(), choice)]);
    let metadata = compile_match(
        &context,
        environment,
        "match choice { Choice.First => 1, Choice.Second => 2, }",
    );

    assert_eq!(
        metadata.shape_dependencies().shape_input_ids(),
        [InputId::new(0)]
    );
    assert_eq!(
        metadata.leaf_dependencies().leaf_input_ids(),
        [InputId::new(0)]
    );
}

fn context(variants: &[(&str, bool)]) -> (ExpressionContext, ValueType) {
    let variants: Vec<_> = variants
        .iter()
        .enumerate()
        .map(|(index, (name, payload))| {
            let fields = payload
                .then(|| {
                    vec![FieldDefinition::new(
                        FieldIndex::new(0),
                        "value",
                        PrimitiveType::Integer.into(),
                    )]
                })
                .unwrap_or_default();
            EnumVariantDefinition::new(VariantIndex::new(index.try_into().unwrap()), *name, fields)
        })
        .collect::<Vec<_>>();
    let definition = Arc::new(TypeDefinition::new(
        "types.veac",
        "Choice",
        TypeDefinitionKind::Enum(EnumDefinition::new(variants)),
    ));
    let choice = ValueType::nominal(definition.type_ref().clone());
    let mut registry = TypeRegistryBuilder::new();
    registry.insert(Arc::clone(&definition)).unwrap();
    registry
        .bind("Choice", definition.type_ref().clone())
        .unwrap();
    (
        ExpressionContext::empty().with_types(Arc::new(registry.finish().unwrap())),
        choice,
    )
}

fn compile_match(
    context: &ExpressionContext,
    environment: TypeEnvironment,
    source: &str,
) -> CoreValueMetadata {
    let expression = compile_expression(source, &environment, context).unwrap();
    result_metadata(&expression).clone()
}

fn result_metadata(expression: &CompiledExpression) -> &CoreValueMetadata {
    let value = match expression.core().blocks().last().unwrap().terminator() {
        CoreTerminator::Return { value, .. } => *value,
        _ => unreachable!("match expression returns from its join block"),
    };
    expression
        .core()
        .blocks()
        .iter()
        .find_map(|block| {
            block
                .parameters()
                .iter()
                .find(|parameter| parameter.id() == value)
                .map(|parameter| parameter.metadata())
                .or_else(|| {
                    block
                        .instructions()
                        .iter()
                        .find(|instruction| instruction.id() == value)
                        .map(|instruction| instruction.metadata())
                })
        })
        .unwrap()
}
