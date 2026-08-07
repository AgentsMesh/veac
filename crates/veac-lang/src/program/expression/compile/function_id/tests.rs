use super::identity;
use crate::program::expression::{
    FunctionDefinition, FunctionOrigin, FunctionParameter, ValueType,
};

fn definition(value_type: &str) -> FunctionDefinition {
    let value_type = ValueType::parse(value_type).unwrap();
    FunctionDefinition::new(
        "typed",
        vec![FunctionParameter::new("value", value_type.clone())],
        value_type,
        "{value}",
    )
    .with_origin(FunctionOrigin::new("types.veac", 0..7))
}

#[test]
fn identity_type_encoding_distinguishes_every_constructor() {
    let types = [
        "int",
        "scalar",
        "time",
        "length",
        "percent",
        "angle",
        "text",
        "color",
        "bool",
        "identifier",
        "list<int>",
        "range<int>",
        "map<text, int>",
        "map<identifier, int>",
        "(int, int)",
        "fn(int) -> int effect pure",
    ];
    let identities = types
        .iter()
        .map(|value_type| identity(&definition(value_type)))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(identities.len(), types.len());
}

#[test]
fn identity_type_encoding_is_structural_and_stable() {
    let nested = definition("fn(list<int>, (time, scalar)) -> map<text, bool> effect pure");
    assert_eq!(identity(&nested), identity(&nested.clone()));
    assert_ne!(
        identity(&nested),
        identity(&definition(
            "fn(list<int>, (scalar, time)) -> map<text, bool> effect pure"
        ))
    );
}
