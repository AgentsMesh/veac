use std::fmt::Write as _;

use crate::program::expression::{
    compile_functions, ExpressionContext, FunctionDefinition, FunctionEffect, FunctionParameter,
    PrimitiveType, ValueType,
};

pub(super) fn depth_functions(count: usize) -> ExpressionContext {
    let integer = ValueType::primitive(PrimitiveType::Integer);
    let callback = ValueType::function(Vec::new(), integer.clone(), FunctionEffect::Pure).unwrap();
    let definitions = (0..count)
        .map(|index| {
            let body = if index + 1 == count {
                "{ callback() }".to_owned()
            } else {
                format!("{{ f{}(callback) }}", index + 1)
            };
            FunctionDefinition::new(
                format!("f{index}"),
                vec![FunctionParameter::new("callback", callback.clone())],
                integer.clone(),
                body,
            )
        })
        .collect::<Vec<_>>();
    compile_functions(&ExpressionContext::empty(), &definitions).unwrap()
}

pub(super) fn capturing_source(count: usize) -> String {
    let mut bindings = String::new();
    for index in 0..count {
        write!(bindings, "let capture{index} = {index};").unwrap();
    }
    let values = (0..count)
        .map(|index| format!("capture{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {bindings} fn() -> list<int> effect pure {{ [{values}] }} }}")
}
