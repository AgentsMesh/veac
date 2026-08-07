use super::super::{
    compile_functions, compile_functions_bounded, ExpressionContext, FunctionDefinition,
    PrimitiveType, ValueType,
};

#[test]
fn bounded_compilation_accepts_the_exact_local_hir_payload() {
    let definitions = [
        function("helper", "1.0"),
        function("caller", "helper() + 1.0"),
    ];
    let compiled = compile_functions(&ExpressionContext::empty(), &definitions).unwrap();
    let exact = definitions
        .iter()
        .map(|definition| {
            compiled
                .functions()
                .lookup(&definition.name)
                .unwrap()
                .retained_bytes()
                .unwrap()
        })
        .sum();

    compile_functions_bounded(&ExpressionContext::empty(), &definitions, exact).unwrap();
    let error = compile_functions_bounded(&ExpressionContext::empty(), &definitions, exact - 1)
        .expect_err("one byte below the exact Core payload must fail");
    assert_eq!(error.code(), "EXPRESSION_RETAINED_LIMIT");
    assert_eq!(error.function_name(), Some("caller"));
}

#[test]
fn imported_hir_is_shared_instead_of_recharged() {
    let imported_definition = function("base", "1.0");
    let imported = compile_functions(&ExpressionContext::empty(), &[imported_definition]).unwrap();
    let local = [function("derived", "base() + 1.0")];
    let compiled = compile_functions(&imported, &local).unwrap();
    let exact = compiled
        .functions()
        .lookup("derived")
        .unwrap()
        .retained_bytes()
        .unwrap();

    compile_functions_bounded(&imported, &local, exact).unwrap();
    let error = compile_functions_bounded(&imported, &local, exact - 1).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_RETAINED_LIMIT");
}

#[test]
fn retained_budget_is_applied_after_two_phase_function_typing() {
    let definitions = [
        function("first", "1.0"),
        FunctionDefinition::new(
            "later",
            vec![],
            ValueType::primitive(PrimitiveType::Time),
            "{1}",
        ),
    ];
    let error =
        compile_functions_bounded(&ExpressionContext::empty(), &definitions, 0).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_RETURN_TYPE");
    assert_eq!(error.function_name(), Some("later"));
}

fn function(name: &str, body: &str) -> FunctionDefinition {
    FunctionDefinition::new(
        name,
        vec![],
        ValueType::primitive(PrimitiveType::Scalar),
        format!("{{{body}}}"),
    )
}
