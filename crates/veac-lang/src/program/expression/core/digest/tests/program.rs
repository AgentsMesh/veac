use super::super::closure_digest;
use crate::program::expression::{
    compile_expression, ExpressionContext, PrimitiveType, TypeEnvironment,
};

#[test]
fn program_digest_covers_maps_control_flow_closures_and_external_inputs() {
    let types = [("outside".to_owned(), PrimitiveType::Integer.into())]
        .into_iter()
        .collect::<TypeEnvironment>();
    let sources = [
        "#{\"a\": 1, \"b\": 2}",
        "if true { 1 } else { 2 }",
        "fn(value: int) -> int effect pure { value + 1 }",
    ];
    let outputs = sources
        .iter()
        .map(|source| {
            let program = compile_expression(source, &types, &ExpressionContext::empty())
                .unwrap()
                .core()
                .clone();
            closure_digest(
                &[],
                &[],
                &[],
                crate::program::expression::FunctionEffect::Pure,
                false,
                &program,
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(outputs.len(), sources.len());

    let program = compile_expression("outside", &types, &ExpressionContext::empty())
        .unwrap()
        .core()
        .clone();
    assert_ne!(
        closure_digest(
            &[PrimitiveType::Integer.into()],
            &[crate::program::expression::Stage::Const],
            &[],
            crate::program::expression::FunctionEffect::Pure,
            false,
            &program,
        ),
        closure_digest(
            &[],
            &[],
            &[PrimitiveType::Integer.into()],
            crate::program::expression::FunctionEffect::Pure,
            true,
            &program,
        )
    );
}
