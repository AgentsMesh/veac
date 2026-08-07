use super::super::super::CoreInstructionKind;
use crate::program::expression::{
    compile_functions, compile_functions_bounded, DependencyMask, ExpressionContext,
    FunctionDefinition, FunctionParameter, PrimitiveType, ValueType,
};

#[test]
fn multi_slot_parameter_dependencies_are_charged_at_the_exact_budget() {
    let definition = FunctionDefinition::new(
        "update",
        vec![parameter("first"), parameter("second")],
        ValueType::primitive(PrimitiveType::Integer),
        concat!(
            "{ var first_local = first; var second_local = second; ",
            "set first_local = first_local + second_local; first_local }"
        ),
    );
    let definitions = [definition];
    let context = ExpressionContext::empty();
    let compiled = compile_functions(&context, &definitions).unwrap();
    let function = compiled.functions().lookup("update").unwrap();
    let unit = DependencyMask::parameter_shape(0)
        .retained_payload_bytes()
        .unwrap();

    assert_eq!(function.body().local_slots().len(), 2);
    for slot in function.body().local_slots() {
        assert_carries_both_parameters(slot.metadata());
        assert_eq!(metadata_dependency_bytes(slot.metadata()), 4 * unit);
    }

    let local_instructions = function
        .body()
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .filter(|instruction| {
            matches!(
                instruction.kind(),
                CoreInstructionKind::LocalInit { .. }
                    | CoreInstructionKind::LocalSet { .. }
                    | CoreInstructionKind::LocalGet { .. }
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(local_instructions.len(), 6);
    for instruction in local_instructions {
        assert_carries_both_parameters(instruction.metadata());
        assert_eq!(metadata_dependency_bytes(instruction.metadata()), 4 * unit);
    }

    let exact = function.retained_bytes().unwrap();
    compile_functions_bounded(&context, &definitions, exact).unwrap();
    let error = compile_functions_bounded(&context, &definitions, exact - 1).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_RETAINED_LIMIT");
}

fn parameter(name: &str) -> FunctionParameter {
    FunctionParameter::new(name, ValueType::primitive(PrimitiveType::Integer))
}

fn metadata_dependency_bytes(metadata: &super::super::super::CoreValueMetadata) -> usize {
    metadata
        .shape_dependencies()
        .retained_payload_bytes()
        .unwrap()
        + metadata
            .leaf_dependencies()
            .retained_payload_bytes()
            .unwrap()
}

fn assert_carries_both_parameters(metadata: &super::super::super::CoreValueMetadata) {
    for index in 0..2 {
        assert!(metadata
            .shape_dependencies()
            .depends_on_parameter_shape(index));
        assert!(metadata
            .leaf_dependencies()
            .depends_on_parameter_leaf(index));
    }
}
