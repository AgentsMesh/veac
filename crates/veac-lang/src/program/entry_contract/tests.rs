use std::sync::Arc;

use super::*;
use crate::program::expression::{Effect, PrimitiveType, Value};
use crate::program::{prepare_host_source, EntryParameterContract};

const SOURCE: &str = r#"
struct Manifest { id: identifier, values: list<int>, }
fn workspace() -> Manifest {
    Manifest { id: identifier("demo"), values: [1, 2, 3], }
}
"#;

fn contract() -> EntryContract {
    EntryContract::new("workspace", EntryValueType::nominal("Manifest"))
}

#[test]
fn pure_nominal_host_entry_executes_to_an_inspectable_value() {
    let prepared = prepare_host_source(SOURCE, &contract()).unwrap();
    assert_eq!(prepared.root_module(), "main.veac");
    assert_eq!(prepared.sources().len(), 1);
    let evaluated = prepared.execute(&[]).unwrap();
    let Value::Struct(value) = evaluated.value() else {
        panic!("entry must return its nominal struct");
    };
    assert_eq!(value.fields()[0], Value::Identifier(Arc::from("demo")));
    let definition = evaluated
        .type_registry()
        .definition(value.type_id())
        .unwrap();
    assert_eq!(definition.declared_name(), "Manifest");
}

#[test]
fn host_contract_reports_missing_duplicate_and_arity_errors() {
    let cases = [
        ("fn other() -> int { 1 }", "PROGRAM_ENTRY_MISSING"),
        (
            "fn workspace() -> int { 1 } fn workspace() -> int { 2 }",
            "PROGRAM_ENTRY_DUPLICATE",
        ),
        (
            "fn workspace(value: int) -> int { value }",
            "PROGRAM_ENTRY_ARITY",
        ),
    ];
    for (source, code) in cases {
        let error = prepare_host_source(source, &contract()).unwrap_err();
        assert_eq!(error.as_slice()[0].code, code);
    }
}

#[test]
fn host_contract_rejects_inputs_temporal_and_wrong_result_types() {
    let input = "input parameter locale: text; fn workspace() -> text { locale }";
    assert_eq!(
        prepare_host_source(
            input,
            &EntryContract::new("workspace", EntryValueType::Primitive(PrimitiveType::Text))
        )
        .unwrap_err()
        .as_slice()[0]
            .code,
        "PROGRAM_ENTRY_INPUTS"
    );
    let temporal = format!(
        "{SOURCE}\nanimate visual-opacity on clip(@demo, @main, @visual, @first) {{ progress }}"
    );
    assert_eq!(
        prepare_host_source(&temporal, &contract())
            .unwrap_err()
            .as_slice()[0]
            .code,
        "PROGRAM_ENTRY_TEMPORAL"
    );
    let wrong = "fn workspace() -> int { 1 }";
    assert_eq!(
        prepare_host_source(wrong, &contract())
            .unwrap_err()
            .as_slice()[0]
            .code,
        "PROGRAM_ENTRY_SIGNATURE"
    );
}

#[test]
fn effect_ceiling_and_arguments_are_enforced() {
    let emitting = r#"
fn workspace(context: Context) -> Project {
    project(identifier("host"), project_settings(60))
}
"#;
    let strict = EntryContract::new(
        "workspace",
        EntryValueType::Domain(crate::program::DomainType::Project),
    )
    .with_parameter(EntryParameterContract::new(
        "context",
        EntryValueType::Domain(crate::program::DomainType::Context),
    ));
    assert_eq!(
        prepare_host_source(emitting, &strict)
            .unwrap_err()
            .as_slice()[0]
            .code,
        "PROGRAM_ENTRY_EFFECT"
    );
    let emitting_contract = strict.with_effect(Effect::GraphEmit);
    assert!(prepare_host_source(emitting, &emitting_contract).is_ok());

    let source = "fn inspect(value: int) -> int { value + 1 }";
    let argument_contract =
        EntryContract::new("inspect", EntryValueType::Primitive(PrimitiveType::Integer))
            .with_parameter(EntryParameterContract::new(
                "value",
                EntryValueType::Primitive(PrimitiveType::Integer),
            ));
    let prepared = prepare_host_source(source, &argument_contract).unwrap();
    assert_eq!(
        prepared.execute(&[Value::Integer(2)]).unwrap().value(),
        &Value::Integer(3)
    );
    assert!(prepared.execute(&[]).is_err());
}

#[path = "tests/host_prelude.rs"]
mod host_prelude;
#[path = "tests/paths.rs"]
mod paths;
