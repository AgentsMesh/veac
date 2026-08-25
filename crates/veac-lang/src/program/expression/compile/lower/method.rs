use super::Lowerer;
use crate::program::expression::ast::{CallArgument, Expression};
use crate::program::expression::hir::{TypedMethodCall, TypedNodeKind};
use crate::program::expression::{ExpressionError, ValueType, ValueTypeKind};

impl Lowerer<'_> {
    pub(super) fn member_call(
        &mut self,
        receiver: &Expression,
        name: &str,
        name_span: &std::ops::Range<usize>,
        arguments: &[CallArgument],
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let receiver = self.lower(receiver)?;
        if let Some(receiver_type) = receiver.value_type.as_domain() {
            return self.domain_method(receiver, receiver_type, name, arguments, expression);
        }
        let ValueTypeKind::Nominal(reference) = receiver.value_type.kind() else {
            return Err(ExpressionError::new(
                "EXPRESSION_METHOD_RECEIVER_TYPE",
                format!(
                    "method `{name}` requires a verified nominal receiver, found {}",
                    receiver.value_type
                ),
                receiver.span.clone(),
            ));
        };
        if self.types.definition(reference.id()).is_none() {
            return Err(ExpressionError::new(
                "EXPRESSION_NOMINAL_UNKNOWN_TYPE",
                format!("method receiver type `{reference}` is outside the verified registry"),
                receiver.span.clone(),
            ));
        }
        let Some(definition) = self.methods.lookup(reference.id(), name) else {
            return self
                .function_field_or_unknown(receiver, name, name_span, arguments, expression);
        };
        let signature = definition.signature().clone();
        let parameters = signature
            .explicit_parameters()
            .iter()
            .map(|parameter| super::call::DeclaredParameter {
                name: &parameter.name,
                value_type: parameter.value_type.clone(),
                default: parameter.default().map(|value| value.thunk()),
            })
            .collect::<Vec<_>>();
        let lowered = self.bind_call_arguments(name, arguments, &parameters, 0, expression)?;
        Ok((
            TypedNodeKind::MethodCall(TypedMethodCall {
                target: signature.function_id(),
                receiver: Box::new(receiver),
                arguments: lowered.explicit,
                defaults: lowered.defaults,
            }),
            signature.return_type().clone(),
        ))
    }

    fn function_field_or_unknown(
        &mut self,
        receiver: crate::program::expression::hir::TypedNode,
        name: &str,
        name_span: &std::ops::Range<usize>,
        arguments: &[CallArgument],
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        match self.projected_node(receiver, name, name_span, expression) {
            Ok(callee) => self.invoke(callee, arguments, expression),
            Err(error)
                if matches!(
                    error.code(),
                    "EXPRESSION_UNKNOWN_FIELD" | "EXPRESSION_FIELD_TYPE"
                ) =>
            {
                Err(ExpressionError::new(
                    "EXPRESSION_UNKNOWN_METHOD",
                    format!("receiver has no method `{name}`"),
                    name_span.clone(),
                ))
            }
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests;
