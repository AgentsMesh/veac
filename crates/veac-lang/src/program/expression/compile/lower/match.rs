use std::collections::BTreeMap;

use super::scope::ScopeBinding;
use super::Lowerer;
use crate::program::expression::ast::{Expression, MatchArm};
use crate::program::expression::hir::{
    LocalId, TypedBlock, TypedMatchArm, TypedNode, TypedNodeKind, TypedPatternBinding,
};
use crate::program::expression::{ExpressionError, ValueType, ValueTypeKind};
use crate::program::{FieldDefinition, TypeDefinitionKind, VariantIndex};

mod resolve;

pub(super) struct ResolvedBinding {
    pub name: String,
    pub field: FieldDefinition,
}

pub(super) struct ResolvedArm<'a> {
    pub variant: VariantIndex,
    pub bindings: Vec<ResolvedBinding>,
    pub body: &'a Expression,
    pub span: std::ops::Range<usize>,
}

impl Lowerer<'_> {
    pub(super) fn match_expression(
        &mut self,
        scrutinee: &Expression,
        arms: &[MatchArm],
        expected: Option<&ValueType>,
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let scrutinee = self.lower(scrutinee)?;
        let ValueTypeKind::Nominal(reference) = scrutinee.value_type.kind() else {
            return Err(match_type(&scrutinee, expression));
        };
        let reference = reference.clone();
        let definition = self.types.definition(reference.id()).ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_NOMINAL_UNKNOWN_TYPE",
                format!("match type `{reference}` is outside the verified registry"),
                scrutinee.span.clone(),
            )
        })?;
        let TypeDefinitionKind::Enum(layout) = definition.kind().clone() else {
            return Err(match_type(&scrutinee, expression));
        };
        let resolved = self.resolve_match_arms(&reference, &layout, arms, expression)?;
        let lowered = self.lower_match_arms(&resolved, expected)?;
        let value_type = lowered[0].body.result.value_type.clone();
        Ok((
            TypedNodeKind::Match {
                scrutinee: Box::new(scrutinee),
                arms: lowered,
            },
            value_type,
        ))
    }

    fn lower_match_arms(
        &mut self,
        arms: &[ResolvedArm<'_>],
        expected: Option<&ValueType>,
    ) -> Result<Vec<TypedMatchArm>, ExpressionError> {
        if let Some(expected) = expected {
            return arms
                .iter()
                .map(|arm| self.lower_match_arm(arm, Some(expected)))
                .collect();
        }
        let checkpoint = self.checkpoint();
        match self.lower_match_arm(&arms[0], None) {
            Ok(first) => {
                let inferred = first.body.result.value_type.clone();
                let mut lowered = vec![first];
                for arm in &arms[1..] {
                    lowered.push(self.lower_match_arm(arm, Some(&inferred))?);
                }
                Ok(lowered)
            }
            Err(first_error) if super::binary::needs_context(&first_error) => {
                self.rollback(&checkpoint);
                let inferred = self.infer_match_type(&arms[1..], &checkpoint, first_error)?;
                self.rollback(&checkpoint);
                arms.iter()
                    .map(|arm| self.lower_match_arm(arm, Some(&inferred)))
                    .collect()
            }
            Err(error) => Err(error),
        }
    }

    fn infer_match_type(
        &mut self,
        arms: &[ResolvedArm<'_>],
        checkpoint: &super::scope::Checkpoint,
        fallback: ExpressionError,
    ) -> Result<ValueType, ExpressionError> {
        for arm in arms {
            match self.lower_match_arm(arm, None) {
                Ok(arm) => return Ok(arm.body.result.value_type.clone()),
                Err(error) if super::binary::needs_context(&error) => self.rollback(checkpoint),
                Err(error) => return Err(error),
            }
        }
        Err(fallback)
    }

    fn lower_match_arm(
        &mut self,
        arm: &ResolvedArm<'_>,
        expected: Option<&ValueType>,
    ) -> Result<TypedMatchArm, ExpressionError> {
        let mut scope = BTreeMap::new();
        let mut bindings = Vec::with_capacity(arm.bindings.len());
        for binding in &arm.bindings {
            let id = LocalId::new(self.next_local);
            self.next_local = self
                .next_local
                .checked_add(1)
                .expect("parser node limit bounds local IDs");
            scope.insert(
                binding.name.clone(),
                ScopeBinding::local(id, binding.field.value_type().clone(), self.closures.len()),
            );
            bindings.push(TypedPatternBinding {
                id,
                field: binding.field.index(),
            });
        }
        self.scopes.push(scope);
        let body = self.lower_context(arm.body, expected);
        self.scopes.pop();
        let body = body?;
        if expected.is_some_and(|expected| expected != &body.value_type) {
            return Err(ExpressionError::new(
                "EXPRESSION_MATCH_ARM_TYPE",
                format!(
                    "match arm expects {}, found {}",
                    expected.expect("checked above"),
                    body.value_type
                ),
                arm.span.clone(),
            ));
        }
        Ok(TypedMatchArm {
            variant: arm.variant,
            bindings,
            body: TypedBlock {
                statements: Vec::new(),
                result: Box::new(body),
            },
        })
    }
}

fn match_type(value: &TypedNode, expression: &Expression) -> ExpressionError {
    ExpressionError::new(
        "EXPRESSION_MATCH_TYPE",
        format!("match requires an enum value, found {}", value.value_type),
        expression.span.clone(),
    )
}
