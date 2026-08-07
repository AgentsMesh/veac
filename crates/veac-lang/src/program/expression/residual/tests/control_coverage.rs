use std::collections::BTreeMap;
use std::sync::Arc;

use super::support::*;
use crate::program::expression::{
    compile_temporal_expression, CoreBuildInputId, CoreTemporalInputIdentity, ExpressionContext,
    ResidualBuildBindings, TypeEnvironment, Value, ValueType,
};
use crate::program::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, TypeDefinition,
    TypeDefinitionKind, TypeRegistryBuilder, VariantIndex,
};
use veac_ir::{TemporalType, TemporalValue};

#[test]
fn temporal_branch_join_binds_parameters_before_continuing() {
    let expression = compile(
        "(if condition { 1.0 } else { 2.0 }) + gain",
        &[
            (
                "condition",
                parameter("join_condition", TemporalType::Boolean),
            ),
            ("gain", parameter("join_gain", TemporalType::Scalar)),
        ],
    );
    let result = residualize(&expression, &no_bindings(), "branch_join");
    assert_eq!(
        evaluate(
            &result,
            vec![
                (0, TemporalValue::Boolean { value: false }),
                (1, TemporalValue::Scalar { value: 3.0 }),
            ],
        ),
        TemporalValue::Scalar { value: 5.0 }
    );
}

#[test]
fn purity_proof_walks_nested_branches_and_shared_join_blocks() {
    let expression = compile(
        "(if first { if second { 1.0 } else { 2.0 } } else { 3.0 }) + 1.0",
        &[
            ("first", parameter("first_nested", TemporalType::Boolean)),
            ("second", parameter("second_nested", TemporalType::Boolean)),
        ],
    );
    let result = residualize(&expression, &no_bindings(), "nested_branch");
    assert_eq!(
        evaluate(
            &result,
            vec![
                (0, TemporalValue::Boolean { value: true }),
                (1, TemporalValue::Boolean { value: false }),
            ],
        ),
        TemporalValue::Scalar { value: 3.0 }
    );
}

#[test]
fn concrete_nominal_match_binds_payload_and_purity_walks_all_arms() {
    let integer: ValueType = crate::program::expression::PrimitiveType::Integer.into();
    let definition = Arc::new(TypeDefinition::new(
        "residual-control.veac",
        "ResidualChoice",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![
            EnumVariantDefinition::new(
                VariantIndex::new(0),
                "Ready",
                vec![FieldDefinition::new(FieldIndex::new(0), "value", integer)],
            ),
            EnumVariantDefinition::new(VariantIndex::new(1), "Fallback", Vec::new()),
        ])),
    ));
    let mut registry = TypeRegistryBuilder::new();
    registry.insert(Arc::clone(&definition)).unwrap();
    registry
        .bind(definition.declared_name(), definition.type_ref().clone())
        .unwrap();
    let registry = Arc::new(registry.finish().unwrap());
    let context = ExpressionContext::empty().with_types(Arc::clone(&registry));
    let build = TypeEnvironment::from([(
        "choice".to_owned(),
        ValueType::nominal(definition.type_ref().clone()),
    )]);
    let temporal = BTreeMap::from([(
        "condition".to_owned(),
        CoreTemporalInputIdentity::Parameter {
            parameter_id: veac_ir::TemporalParameterId::new("tpm_match_condition").unwrap(),
            value_type: TemporalType::Boolean,
        },
    )]);
    let expression = compile_temporal_expression(
        "if condition { match choice { ResidualChoice.Ready { value } => value, \
         ResidualChoice.Fallback => 0, } } else { 9 }",
        &build,
        &temporal,
        &context,
    )
    .unwrap();
    let choice = Value::variant(
        registry.as_ref(),
        definition.type_ref().id(),
        VariantIndex::new(0),
        vec![Value::Integer(7)],
    )
    .unwrap();
    let bindings = ResidualBuildBindings::from([(CoreBuildInputId::for_symbol("choice"), choice)]);
    let result = residualize(&expression, &bindings, "nominal_match");
    assert_eq!(
        evaluate(&result, vec![(0, TemporalValue::Boolean { value: true })],),
        TemporalValue::Integer { value: 7 }
    );
}
