use super::super::{ExpressionContext, ExpressionError, ValueType, ValueTypeKind};
use crate::program::TypeRegistry;

pub(super) fn context(value: &ExpressionContext) -> Result<(), ExpressionError> {
    for (_, function) in value.functions().iter() {
        compiled(function, value.types())?;
    }
    for method in value.methods().definitions() {
        let signature_value = method.signature();
        signature(
            signature_value
                .parameters_with_receiver()
                .iter()
                .map(|value| &value.value_type),
            signature_value.return_type(),
            value.types(),
        )?;
        if let Some(function) = value
            .functions()
            .lookup_by_id(signature_value.function_id())
        {
            compiled(function, value.types())?;
        }
    }
    Ok(())
}

fn compiled(
    function: &super::super::CompiledFunction,
    registry: &TypeRegistry,
) -> Result<(), ExpressionError> {
    signature(
        function.parameters().iter().map(|value| &value.value_type),
        function.return_type(),
        registry,
    )?;
    for embedded in function.body().nominal_definitions() {
        let definition = embedded.definition();
        let current = registry
            .definition(definition.type_ref().id())
            .ok_or_else(|| unknown(definition.type_ref().diagnostic_name()))?;
        if current.digest() != definition.digest() {
            return Err(ExpressionError::new(
                "EXPRESSION_NOMINAL_ABI",
                format!(
                    "compiled function `{}` expects nominal layout {} for `{}`, found {}",
                    function.name(),
                    definition.digest(),
                    definition.type_ref(),
                    current.digest()
                ),
                embedded.span(),
            ));
        }
    }
    Ok(())
}

pub(super) fn signature<'a>(
    parameters: impl IntoIterator<Item = &'a ValueType>,
    result: &'a ValueType,
    registry: &TypeRegistry,
) -> Result<(), ExpressionError> {
    parameters
        .into_iter()
        .chain(std::iter::once(result))
        .try_for_each(|value| require(value, registry))
}

pub(super) fn require(value: &ValueType, registry: &TypeRegistry) -> Result<(), ExpressionError> {
    match value.kind() {
        ValueTypeKind::Primitive(_) | ValueTypeKind::Domain(_) => Ok(()),
        ValueTypeKind::Nominal(value) => registry
            .definition(value.id())
            .map(|_| ())
            .ok_or_else(|| unknown(value.diagnostic_name())),
        ValueTypeKind::List(value)
        | ValueTypeKind::Range(value)
        | ValueTypeKind::Map { value, .. } => require(value, registry),
        ValueTypeKind::Tuple(values) => {
            values.iter().try_for_each(|value| require(value, registry))
        }
        ValueTypeKind::Function {
            parameters, result, ..
        } => parameters
            .iter()
            .chain(std::iter::once(result))
            .try_for_each(|value| require(value, registry)),
    }
}

fn unknown(name: &str) -> ExpressionError {
    ExpressionError::new(
        "EXPRESSION_UNKNOWN_TYPE",
        format!("function signature refers to unknown nominal type `{name}`"),
        0..0,
    )
}
