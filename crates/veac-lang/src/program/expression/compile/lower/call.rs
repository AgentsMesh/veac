use super::Lowerer;
use crate::program::expression::ast::Expression;
use crate::program::expression::hir::{CallTarget, TypedNode, TypedNodeKind};
use crate::program::expression::{
    BuiltinFunction, CollectionOperation, ExpressionError, FunctionId, FunctionParameter,
    ValueType, MAX_CALL_ARGUMENTS,
};

mod invoke;

pub(super) use invoke::check_value_argument;

impl Lowerer<'_> {
    pub(super) fn call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        if arguments.len() > MAX_CALL_ARGUMENTS {
            return Err(ExpressionError::new(
                "EXPRESSION_CALL_ARGUMENT_LIMIT",
                format!("function call exceeds the {MAX_CALL_ARGUMENTS} argument limit"),
                expression.span.clone(),
            ));
        }
        if let Some(path) = crate::program::expression::ast::static_path(callee) {
            let name = crate::program::expression::ast::join_path(&path);
            if !self.shadows_static(&path[0].name) {
                if let Some(callee) = self.dynamic_callee(&name, callee)? {
                    return self.invoke(callee, arguments, expression);
                }
                if self.has_named_callable(&name) || self.functions.is_namespace(&path[0].name) {
                    return self.named_call(&name, arguments, expression);
                }
            }
        }
        if let crate::program::expression::ast::ExpressionKind::FieldProject {
            receiver,
            field,
            field_span,
        } = &callee.kind
        {
            return self.member_call(receiver, field, field_span, arguments, expression);
        }
        if let Some(path) = crate::program::expression::ast::static_path(callee) {
            if !self.shadows_static(&path[0].name) {
                return self.named_call(
                    &crate::program::expression::ast::join_path(&path),
                    arguments,
                    expression,
                );
            }
        }
        let callee = self.lower(callee)?;
        self.invoke(callee, arguments, expression)
    }

    fn has_named_callable(&self, name: &str) -> bool {
        self.domain.lookup_function(name).is_some()
            || CollectionOperation::parse(name).is_some()
            || BuiltinFunction::parse(name).is_some()
            || self.signatures.contains_key(name)
            || self.functions.lookup(name).is_some()
    }

    fn named_call(
        &mut self,
        name: &str,
        arguments: &[Expression],
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        if let Some(contract) = self.domain.lookup_function(name).cloned() {
            return self.domain_function(&contract, arguments, expression);
        }
        if let Some(operation) = CollectionOperation::parse(name) {
            return self.collection_call(operation, arguments, expression);
        }
        if let Some(function) = BuiltinFunction::parse(name) {
            let arguments = arguments
                .iter()
                .map(|argument| self.lower(argument))
                .collect::<Result<Vec<_>, _>>()?;
            let types = argument_types(&arguments);
            let value_type = super::typing::builtin(function, &types, expression.span.clone())?;
            return Ok((
                TypedNodeKind::Call {
                    target: CallTarget::Builtin(function),
                    arguments,
                },
                value_type,
            ));
        }
        if let Some(signature) = self.signatures.get(name).cloned() {
            return self.user_call(
                signature.id(),
                signature.name(),
                signature.parameters(),
                signature.return_type(),
                arguments,
                expression,
            );
        }
        let function = self.functions.lookup(name).cloned().ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_UNKNOWN_FUNCTION",
                format!("unknown function `{name}`"),
                expression.span.clone(),
            )
        })?;
        self.user_call(
            function.id(),
            function.name(),
            function.parameters(),
            function.return_type(),
            arguments,
            expression,
        )
    }

    pub(super) fn user_call(
        &mut self,
        id: FunctionId,
        name: &str,
        parameters: &[FunctionParameter],
        return_type: &ValueType,
        arguments: &[Expression],
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        super::typing::require_arity(
            name,
            arguments.len(),
            parameters.len(),
            expression.span.clone(),
        )?;
        let lowered = arguments
            .iter()
            .zip(parameters)
            .enumerate()
            .map(|(index, (argument, parameter))| {
                let value = self.lower_context(argument, Some(&parameter.value_type))?;
                check_value_argument(index, &value, &parameter.value_type, argument)?;
                Ok(value)
            })
            .collect::<Result<Vec<_>, ExpressionError>>()?;
        Ok((
            TypedNodeKind::Call {
                target: CallTarget::User(id),
                arguments: lowered,
            },
            return_type.clone(),
        ))
    }
}

fn argument_types(arguments: &[TypedNode]) -> Vec<ValueType> {
    arguments
        .iter()
        .map(|value| value.value_type.clone())
        .collect()
}
