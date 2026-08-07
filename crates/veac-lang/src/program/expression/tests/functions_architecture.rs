use std::sync::Arc;

use crate::program::expression::{
    compile_expression, compile_functions, CompiledExpression, CoreTerminator, CoreValueMetadata,
    Effect, ExpressionContext, FunctionDefinition, FunctionOrigin, FunctionParameter, InputId,
    PrimitiveType, Stage, ValueType,
};

fn function(name: &str, parameters: &[&str], body: &str) -> FunctionDefinition {
    FunctionDefinition::new(
        name,
        parameters
            .iter()
            .map(|name| FunctionParameter::new(*name, ValueType::primitive(PrimitiveType::Scalar)))
            .collect(),
        ValueType::primitive(PrimitiveType::Scalar),
        format!("{{{body}}}"),
    )
}

#[test]
fn function_identity_is_stable_while_content_tracks_the_call_graph() {
    let first = function("helper", &[], "1.0").with_origin(FunctionOrigin::new("math.veac", 0..3));
    let edited = function("helper", &[], "2.0").with_origin(FunctionOrigin::new("math.veac", 0..3));
    let first_map = compile_functions(&ExpressionContext::empty(), &[first]).unwrap();
    let edited_map = compile_functions(&ExpressionContext::empty(), &[edited]).unwrap();
    let first = first_map.functions().lookup("helper").unwrap();
    let edited = edited_map.functions().lookup("helper").unwrap();
    assert_eq!(first.id(), edited.id());
    assert_ne!(first.content_digest(), edited.content_digest());

    let caller =
        function("caller", &[], "helper()").with_origin(FunctionOrigin::new("entry.veac", 10..20));
    let first_callers = compile_functions(&first_map, std::slice::from_ref(&caller)).unwrap();
    let edited_callers = compile_functions(&edited_map, &[caller]).unwrap();
    let first_caller = first_callers.functions().lookup("caller").unwrap();
    let edited_caller = edited_callers.functions().lookup("caller").unwrap();
    assert_eq!(first_caller.id(), edited_caller.id());
    assert_ne!(
        first_caller.content_digest(),
        edited_caller.content_digest()
    );

    let other_source =
        function("helper", &[], "1.0").with_origin(FunctionOrigin::new("other.veac", 0..3));
    let other = compile_functions(&ExpressionContext::empty(), &[other_source]).unwrap();
    assert_ne!(first.id(), other.functions().lookup("helper").unwrap().id());
}

#[test]
fn summaries_are_stage_polymorphic_and_parameter_precise() {
    let functions = compile_functions(
        &ExpressionContext::empty(),
        &[
            function("ignore", &["x"], "1.0"),
            function("identity", &["x"], "x"),
            function("mixed", &["x", "unused"], "x + 1.0"),
        ],
    )
    .unwrap();
    let ignore = functions
        .functions()
        .lookup("ignore")
        .unwrap()
        .summary()
        .result();
    assert_eq!(ignore.effect(), Effect::Pure);
    assert_eq!(ignore.leaf_stage(), Stage::Const);
    assert!(ignore.shape_dependencies().is_empty());
    assert!(ignore.leaf_dependencies().is_empty());
    let identity = functions
        .functions()
        .lookup("identity")
        .unwrap()
        .summary()
        .result();
    assert_eq!(identity.leaf_stage(), Stage::Const);
    assert!(identity.shape_dependencies().depends_on_parameter_shape(0));
    assert!(identity.leaf_dependencies().depends_on_parameter_leaf(0));
    let mixed = functions
        .functions()
        .lookup("mixed")
        .unwrap()
        .summary()
        .result();
    assert!(mixed.shape_dependencies().depends_on_parameter_shape(0));
    assert!(mixed.leaf_dependencies().depends_on_parameter_leaf(0));
    assert!(!mixed.shape_dependencies().depends_on_parameter_shape(1));
    assert!(!mixed.leaf_dependencies().depends_on_parameter_leaf(1));

    let types = [(
        "input".to_owned(),
        ValueType::primitive(PrimitiveType::Scalar),
    )]
    .into_iter()
    .collect();
    let ignored = compile_expression("ignore(input)", &types, &functions).unwrap();
    let identity = compile_expression("identity(input)", &types, &functions).unwrap();
    let ignored = result_metadata(&ignored);
    assert_eq!(ignored.shape_stage(), Stage::Const);
    assert_eq!(ignored.leaf_stage(), Stage::Const);
    assert!(ignored.shape_dependencies().is_empty());
    assert!(ignored.leaf_dependencies().is_empty());
    let identity = result_metadata(&identity);
    assert_eq!(identity.shape_stage(), Stage::Build);
    assert_eq!(identity.leaf_stage(), Stage::Build);
    assert_eq!(
        identity.shape_dependencies().shape_input_ids(),
        [InputId::new(0)]
    );
    assert_eq!(
        identity.leaf_dependencies().leaf_input_ids(),
        [InputId::new(0)]
    );
}

#[test]
fn function_registry_ownership_has_no_arc_cycle() {
    let functions = compile_functions(
        &ExpressionContext::empty(),
        &[function("identity", &["x"], "x")],
    )
    .unwrap();
    let weak = Arc::downgrade(functions.functions().lookup("identity").unwrap());
    drop(functions);
    assert!(weak.upgrade().is_none());
}

fn result_metadata(expression: &CompiledExpression) -> &CoreValueMetadata {
    let block = expression.core().blocks().last().unwrap();
    match block.terminator() {
        CoreTerminator::Return { value, .. } => block
            .instructions()
            .iter()
            .find(|instruction| instruction.id() == *value)
            .unwrap()
            .metadata(),
        _ => unreachable!("straight call expression returns from its only block"),
    }
}
