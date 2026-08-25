use super::Lowerer;
use crate::program::expression::ast::{CallArgument, Expression};
use crate::program::expression::hir::{
    TypedCallArgument, TypedDomainCall, TypedNode, TypedNodeKind,
};
use crate::program::expression::{ExpressionError, ValueType};
use crate::program::{DomainOperationContract, DomainType};

impl Lowerer<'_> {
    pub(super) fn domain_function(
        &mut self,
        contract: &DomainOperationContract,
        arguments: &[CallArgument],
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        self.typed_domain_call(contract, Vec::new(), arguments, expression)
    }

    pub(super) fn domain_method(
        &mut self,
        receiver: TypedNode,
        receiver_type: DomainType,
        name: &str,
        arguments: &[CallArgument],
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let contract = self
            .domain
            .lookup_method(receiver_type, name)
            .cloned()
            .ok_or_else(|| {
                ExpressionError::new(
                    "EXPRESSION_UNKNOWN_METHOD",
                    format!("{receiver_type} has no method `{name}`"),
                    expression.span.clone(),
                )
            })?;
        self.typed_domain_call(
            &contract,
            vec![TypedCallArgument {
                slot: 0,
                value: receiver,
            }],
            arguments,
            expression,
        )
    }

    fn typed_domain_call(
        &mut self,
        contract: &DomainOperationContract,
        mut operands: Vec<TypedCallArgument>,
        arguments: &[CallArgument],
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let offset = operands.len();
        let parameters = contract.operands()[offset..]
            .iter()
            .map(|specification| super::call::DeclaredParameter {
                name: specification.name(),
                value_type: specification.shape().value_type(),
                default: None,
            })
            .collect::<Vec<_>>();
        let bound =
            self.bind_call_arguments(contract.name(), arguments, &parameters, offset, expression)?;
        debug_assert!(bound.defaults.is_empty());
        operands.extend(bound.explicit);
        Ok((
            TypedNodeKind::DomainCall(TypedDomainCall {
                opcode: contract.id().opcode(),
                operands,
            }),
            contract.result().value_type(),
        ))
    }
}
