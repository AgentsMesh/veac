use super::Lowerer;
use crate::program::expression::ast::Expression;
use crate::program::expression::hir::{TypedDomainCall, TypedNode, TypedNodeKind};
use crate::program::expression::{ExpressionError, ValueType};
use crate::program::{DomainOperationContract, DomainType};

impl Lowerer<'_> {
    pub(super) fn domain_function(
        &mut self,
        contract: &DomainOperationContract,
        arguments: &[Expression],
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        super::typing::require_arity(
            contract.name(),
            arguments.len(),
            contract.operands().len(),
            expression.span.clone(),
        )?;
        self.typed_domain_call(contract, Vec::new(), arguments, expression)
    }

    pub(super) fn domain_method(
        &mut self,
        receiver: TypedNode,
        receiver_type: DomainType,
        name: &str,
        arguments: &[Expression],
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
        super::typing::require_arity(
            name,
            arguments.len(),
            contract.operands().len() - 1,
            expression.span.clone(),
        )?;
        self.typed_domain_call(&contract, vec![receiver], arguments, expression)
    }

    fn typed_domain_call(
        &mut self,
        contract: &DomainOperationContract,
        mut operands: Vec<TypedNode>,
        arguments: &[Expression],
        _expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let offset = operands.len();
        for (index, (argument, specification)) in arguments
            .iter()
            .zip(&contract.operands()[offset..])
            .enumerate()
        {
            let expected = specification.shape().value_type();
            let value = self.lower_context(argument, Some(&expected))?;
            super::call::check_value_argument(index, &value, &expected, argument)?;
            operands.push(value);
        }
        Ok((
            TypedNodeKind::DomainCall(TypedDomainCall {
                opcode: contract.id().opcode(),
                operands,
            }),
            contract.result().value_type(),
        ))
    }
}
