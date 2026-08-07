use super::*;

#[test]
fn exact_boundaries_succeed_and_failed_charges_do_not_commit() {
    let value = Value::Text("payload".into());
    let exact = ENTRY_BYTES + "name".len() + value.retained_bytes();
    let mut budget = Budget {
        bytes: 0,
        limit: exact,
    };
    budget
        .value("main.veac", "name", &value, Span::default())
        .unwrap();
    assert_eq!(budget.bytes, exact);

    let error = budget
        .alias("main.veac", "next", Span::default())
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_RETAINED_SCOPE_BUDGET");
    assert_eq!(budget.bytes, exact);
}

#[test]
fn aliases_charge_entries_without_recharging_shared_payloads() {
    let value = Value::Text("large-payload".repeat(100).into());
    let first = ENTRY_BYTES + "source".len() + value.retained_bytes();
    let aliases = ["left.source", "right.source"];
    let total = aliases
        .iter()
        .fold(first, |bytes, name| bytes + ENTRY_BYTES + name.len());
    let mut budget = Budget {
        bytes: 0,
        limit: total,
    };
    budget
        .value("module.veac", "source", &value, Span::default())
        .unwrap();
    for name in aliases {
        budget.alias("main.veac", name, Span::default()).unwrap();
    }
    assert_eq!(budget.bytes, total);
}

#[test]
fn arithmetic_overflow_fails_closed() {
    let mut budget = Budget {
        bytes: usize::MAX,
        limit: usize::MAX,
    };
    let error = budget
        .alias("main.veac", "value", Span::default())
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_RETAINED_SCOPE_BUDGET");
    assert_eq!(budget.bytes, usize::MAX);
}

#[test]
fn function_payload_limit_reserves_every_map_entry_and_name() {
    use crate::program::expression::{FunctionDefinition, PrimitiveType, ValueType};

    let definitions = [
        FunctionDefinition::new(
            "first",
            vec![],
            ValueType::primitive(PrimitiveType::Scalar),
            "1",
        ),
        FunctionDefinition::new(
            "second",
            vec![],
            ValueType::primitive(PrimitiveType::Scalar),
            "2",
        ),
    ];
    let budget = Budget {
        bytes: 11,
        limit: 1_000,
    };
    let overhead = ENTRY_BYTES * 2 + "first".len() + "second".len();
    assert_eq!(budget.function_payload_limit(&definitions), 989 - overhead);
}

#[test]
fn compiled_functions_are_charged_once_and_aliases_share_the_hir() {
    use crate::program::expression::{
        compile_functions, ExpressionContext, FunctionDefinition, PrimitiveType, ValueType,
    };
    use crate::program::parser;

    let definition = FunctionDefinition::new(
        "identity",
        vec![],
        ValueType::primitive(PrimitiveType::Text),
        "{\"value\"}",
    );
    let compiled = compile_functions(&ExpressionContext::empty(), &[definition]).unwrap();
    let functions = compiled.functions();
    let function = functions.lookup("identity").unwrap();
    let first = ENTRY_BYTES + "identity".len() + function.retained_bytes().unwrap();
    let total = first + ENTRY_BYTES + "timing.identity".len();
    let file = parser::parse(
        "timing.veac",
        "module { fn identity() -> text { \"value\" } }",
    )
    .unwrap();
    let mut budget = Budget {
        bytes: 0,
        limit: total,
    };
    budget
        .functions(&file, functions, &Default::default())
        .unwrap();
    budget
        .alias("main.veac", "timing.identity", Span::default())
        .unwrap();
    assert_eq!(budget.bytes, total);
}

#[test]
fn function_batches_commit_atomically() {
    use crate::program::expression::{
        compile_functions, ExpressionContext, FunctionDefinition, PrimitiveType, ValueType,
    };
    use crate::program::parser;

    let source = "module { fn first() -> scalar { 1 } fn second() -> scalar { 2 } }";
    let file = parser::parse("functions.veac", source).unwrap();
    let definitions = [
        FunctionDefinition::new(
            "first",
            vec![],
            ValueType::primitive(PrimitiveType::Scalar),
            "{1.0}",
        ),
        FunctionDefinition::new(
            "second",
            vec![],
            ValueType::primitive(PrimitiveType::Scalar),
            "{2.0}",
        ),
    ];
    let compiled = compile_functions(&ExpressionContext::empty(), &definitions).unwrap();
    let functions = compiled.functions();
    let first = functions.lookup("first").unwrap();
    let first_entry = ENTRY_BYTES + "first".len() + first.retained_bytes().unwrap();
    let mut budget = Budget {
        bytes: 0,
        limit: first_entry,
    };

    let error = budget
        .functions(&file, functions, &Default::default())
        .unwrap_err();
    assert_eq!(error.code, "PROGRAM_RETAINED_SCOPE_BUDGET");
    assert_eq!(budget.bytes, 0);
}

#[path = "tests/methods.rs"]
mod methods;
