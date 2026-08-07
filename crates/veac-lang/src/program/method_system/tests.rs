use std::sync::Arc;

use super::*;
use crate::program::expression::{FunctionOrigin, FunctionParameter, PrimitiveType, ValueType};
use crate::program::{
    StructDefinition, TypeDefinition, TypeDefinitionKind, TypeRegistry, TypeRegistryBuilder,
};

fn types(source: &str, name: &str) -> TypeRegistry {
    let definition = Arc::new(TypeDefinition::new(
        source,
        name,
        TypeDefinitionKind::Struct(StructDefinition::new(Vec::new())),
    ));
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::clone(&definition)).unwrap();
    builder.bind(name, definition.type_ref().clone()).unwrap();
    builder.finish().unwrap()
}

fn method(
    types: &TypeRegistry,
    name: &str,
    visibility: MethodVisibility,
    body: bool,
) -> Arc<MethodDefinition> {
    let receiver = types.resolve("Brand").unwrap().clone();
    let signature = MethodSignature::new(
        receiver,
        name,
        vec![FunctionParameter::new(
            "value",
            ValueType::primitive(PrimitiveType::Text),
        )],
        ValueType::primitive(PrimitiveType::Text),
    );
    let value = MethodDefinition::new(signature, "types.veac", visibility);
    Arc::new(if body {
        value.with_body(MethodBody::new(
            r#"{ "ok" }"#,
            FunctionOrigin::new("types.veac", 10..20),
        ))
    } else {
        value
    })
}

#[test]
fn signature_places_nominal_self_at_core_argument_zero() {
    let types = types("types.veac", "Brand");
    let value = method(&types, "title", MethodVisibility::Private, true);
    let signature = value.signature();
    assert_eq!(signature.parameters_with_receiver()[0].name, "self");
    assert_eq!(signature.explicit_parameters()[0].name, "value");
    assert_eq!(signature.parameters_with_receiver().len(), 2);
    assert_eq!(value.body().unwrap().origin().body_span(), 10..20);
}

#[test]
fn alias_spelling_does_not_change_method_identity() {
    let types = types("types.veac", "Brand");
    let receiver = types.resolve("Brand").unwrap();
    let result = ValueType::primitive(PrimitiveType::Text);
    let first = MethodSignature::new(receiver.clone(), "title", Vec::new(), result.clone());
    let alias = MethodSignature::new(
        receiver.with_diagnostic_name("types.Brand"),
        "title",
        Vec::new(),
        result,
    );
    assert_eq!(first.function_id(), alias.function_id());
}

#[test]
fn identity_frames_every_value_type_shape_and_discriminator() {
    use crate::program::DomainType;

    let types = types("types.veac", "Brand");
    let receiver = types.resolve("Brand").unwrap().clone();
    let primitives = [
        PrimitiveType::Integer,
        PrimitiveType::Scalar,
        PrimitiveType::Time,
        PrimitiveType::Length,
        PrimitiveType::Percent,
        PrimitiveType::Angle,
        PrimitiveType::Text,
        PrimitiveType::Color,
        PrimitiveType::Boolean,
        PrimitiveType::Identifier,
    ];
    let mut parameters: Vec<_> = primitives
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            FunctionParameter::new(format!("primitive_{index}"), ValueType::primitive(value))
        })
        .collect();
    let text = ValueType::primitive(PrimitiveType::Text);
    let integer = ValueType::primitive(PrimitiveType::Integer);
    parameters.extend([
        FunctionParameter::new("domain", ValueType::domain(DomainType::Sequence)),
        FunctionParameter::new("nominal", ValueType::nominal(receiver.clone())),
        FunctionParameter::new("list", ValueType::list(text.clone()).unwrap()),
        FunctionParameter::new("range", ValueType::range(integer.clone()).unwrap()),
        FunctionParameter::new(
            "text_map",
            ValueType::map(text.clone(), integer.clone()).unwrap(),
        ),
        FunctionParameter::new(
            "identifier_map",
            ValueType::map(
                ValueType::primitive(PrimitiveType::Identifier),
                text.clone(),
            )
            .unwrap(),
        ),
        FunctionParameter::new(
            "tuple",
            ValueType::tuple(vec![integer.clone(), text.clone()]).unwrap(),
        ),
        FunctionParameter::new(
            "callable",
            ValueType::function(
                vec![text.clone()],
                integer.clone(),
                crate::program::expression::FunctionEffect::Pure,
            )
            .unwrap(),
        ),
    ]);
    let first = MethodSignature::new(receiver.clone(), "all", parameters.clone(), text.clone());
    let same = MethodSignature::new(receiver.clone(), "all", parameters.clone(), text.clone());
    let changed = MethodSignature::new(receiver, "all", parameters, integer);
    assert_eq!(first.function_id(), same.function_id());
    assert_ne!(first.function_id(), changed.function_id());

    let definition = Arc::new(MethodDefinition::new(
        first,
        "types.veac",
        MethodVisibility::Private,
    ));
    let mut methods = MethodRegistryBuilder::new();
    methods.insert(definition, &types).unwrap();
    assert!(methods.finish().retained_bytes() > 0);
}

#[test]
fn admission_rejects_reserved_duplicate_and_excess_parameter_names() {
    use crate::program::expression::MAX_FUNCTION_PARAMETERS;

    let types = types("types.veac", "Brand");
    let receiver = types.resolve("Brand").unwrap().clone();
    let text = ValueType::primitive(PrimitiveType::Text);
    let cases = [
        vec![FunctionParameter::new("self", text.clone())],
        vec![
            FunctionParameter::new("same", text.clone()),
            FunctionParameter::new("same", text.clone()),
        ],
        (0..MAX_FUNCTION_PARAMETERS)
            .map(|index| FunctionParameter::new(format!("p{index}"), text.clone()))
            .collect(),
    ];
    for parameters in cases {
        let signature = MethodSignature::new(
            receiver.clone(),
            "invalid_parameters",
            parameters,
            text.clone(),
        );
        let definition = Arc::new(MethodDefinition::new(
            signature,
            "types.veac",
            MethodVisibility::Private,
        ));
        assert_eq!(
            MethodRegistryBuilder::new()
                .insert(definition, &types)
                .unwrap_err()
                .code(),
            "METHOD_SIGNATURE"
        );
    }
}

#[path = "tests/error.rs"]
mod error;
#[path = "tests/registry.rs"]
mod registry;
