use std::collections::BTreeMap;

use super::scope::{ClosureContext, ScopeBinding};
use super::Lowerer;
use crate::program::expression::ast::{Expression, TemporalAttachment};
use crate::program::expression::hir::{
    TypedClosureParameter, TypedNode, TypedNodeKind, TypedTemporalAttach,
};
use crate::program::expression::{
    ExpressionError, Stage, TemporalAttachmentKind, TemporalOwnerKind, ValueType,
};

impl Lowerer<'_> {
    pub(super) fn temporal_attachment(
        &mut self,
        attachment: &TemporalAttachment,
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let kind = TemporalAttachmentKind::from_parts(attachment.property, attachment.target)
            .ok_or_else(|| {
                ExpressionError::new(
                    "EXPRESSION_TEMPORAL_TARGET_PROPERTY",
                    format!(
                        "property `{}` cannot target `{}`",
                        attachment.property.source_name(),
                        attachment.target.source_name()
                    ),
                    expression.span.clone(),
                )
            })?;
        let expected = 1 + kind.selector_types().len();
        super::typing::require_arity(
            attachment.target.source_name(),
            attachment.arguments.len(),
            expected,
            expression.span.clone(),
        )?;
        let owner_type = kind.owner_type();
        let owner = self.lower_context(&attachment.arguments[0], Some(&owner_type))?;
        require_type(&owner.value_type, &owner_type, &attachment.arguments[0])?;
        let mut selectors = Vec::with_capacity(expected - 1);
        for (argument, primitive) in attachment.arguments[1..].iter().zip(kind.selector_types()) {
            let expected: ValueType = (*primitive).into();
            let value = self.lower_context(argument, Some(&expected))?;
            require_type(&value.value_type, &expected, argument)?;
            selectors.push(value);
        }
        let animation = self.temporal_animation(kind, &attachment.body, expression)?;
        Ok((
            TypedNodeKind::TemporalAttach(TypedTemporalAttach {
                kind,
                owner: Box::new(owner),
                selectors,
                animation: Box::new(animation),
            }),
            owner_type,
        ))
    }

    fn temporal_animation(
        &mut self,
        kind: TemporalAttachmentKind,
        body: &crate::program::expression::ast::Block,
        expression: &Expression,
    ) -> Result<TypedNode, ExpressionError> {
        let (names, types) = animation_parameters(kind);
        let result_type = kind.result_type();
        let value_type = kind.animation_type();
        let checkpoint = self.checkpoint();
        self.closures.push(ClosureContext::default());
        let owner = self.closures.len();
        self.scopes.push(parameter_scope(&names, &types, owner));
        let lowered = self.block(body, Some(&result_type));
        self.scopes.pop();
        let captures = self
            .closures
            .pop()
            .expect("animation closure is active")
            .captures;
        let body = lowered.inspect_err(|_| {
            self.rollback(&checkpoint);
        })?;
        if body.result.value_type != result_type {
            return Err(ExpressionError::new(
                "EXPRESSION_TEMPORAL_RESULT_TYPE",
                format!(
                    "animation expects {result_type}, found {}",
                    body.result.value_type
                ),
                body.result.span.clone(),
            ));
        }
        Ok(TypedNode {
            kind: TypedNodeKind::Closure {
                parameters: types
                    .into_iter()
                    .map(|value_type| TypedClosureParameter {
                        value_type,
                        stage: Stage::Temporal,
                    })
                    .collect(),
                captures,
                body,
                non_escaping: false,
            },
            value_type,
            span: expression.span.clone(),
        })
    }
}

fn animation_parameters(kind: TemporalAttachmentKind) -> (Vec<&'static str>, Vec<ValueType>) {
    let names = match kind.owner() {
        TemporalOwnerKind::Item => vec![
            "sequence_time",
            "clip_time",
            "frame",
            "progress",
            "source_time",
        ],
        TemporalOwnerKind::Apply => vec!["sequence_time", "frame"],
    };
    let parameters = match kind.animation_type().kind() {
        crate::program::expression::ValueTypeKind::Function { parameters, .. } => {
            parameters.to_vec()
        }
        _ => unreachable!("animation type is a function"),
    };
    (names, parameters)
}

fn parameter_scope(
    names: &[&str],
    types: &[ValueType],
    owner: usize,
) -> BTreeMap<String, ScopeBinding> {
    names
        .iter()
        .zip(types)
        .enumerate()
        .map(|(index, (name, value_type))| {
            (
                (*name).to_owned(),
                ScopeBinding::parameter(index, value_type.clone(), owner),
            )
        })
        .collect()
}

fn require_type(
    actual: &ValueType,
    expected: &ValueType,
    expression: &Expression,
) -> Result<(), ExpressionError> {
    (actual == expected).then_some(()).ok_or_else(|| {
        ExpressionError::new(
            "EXPRESSION_TEMPORAL_ATTACHMENT_TYPE",
            format!("temporal attachment expects {expected}, found {actual}"),
            expression.span.clone(),
        )
    })
}
