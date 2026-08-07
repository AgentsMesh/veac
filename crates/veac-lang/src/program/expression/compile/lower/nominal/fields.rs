use std::collections::BTreeSet;

use super::Lowerer;
use crate::program::expression::ast::{Expression, NominalField};
use crate::program::expression::hir::TypedNode;
use crate::program::expression::ExpressionError;
use crate::program::{FieldDefinition, FieldIndex};

impl Lowerer<'_> {
    pub(super) fn lower_nominal_fields(
        &mut self,
        authored: &[NominalField],
        layout: &[FieldDefinition],
        expression: &Expression,
    ) -> Result<Vec<(FieldIndex, TypedNode)>, ExpressionError> {
        let mut seen = BTreeSet::new();
        let mut lowered = Vec::with_capacity(authored.len());
        for field in authored {
            let expected = layout
                .iter()
                .find(|value| value.name() == field.name)
                .ok_or_else(|| {
                    ExpressionError::new(
                        "EXPRESSION_UNKNOWN_FIELD",
                        format!("unknown nominal field `{}`", field.name),
                        field.name_span.clone(),
                    )
                })?;
            if !seen.insert(expected.index()) {
                return Err(ExpressionError::new(
                    "EXPRESSION_DUPLICATE_NOMINAL_FIELD",
                    format!("nominal field `{}` is authored more than once", field.name),
                    field.name_span.clone(),
                ));
            }
            let value = self.lower_context(&field.value, Some(expected.value_type()))?;
            if &value.value_type != expected.value_type() {
                return Err(ExpressionError::new(
                    "EXPRESSION_NOMINAL_FIELD_TYPE",
                    format!(
                        "field `{}` expects {}, found {}",
                        field.name,
                        expected.value_type(),
                        value.value_type
                    ),
                    field.value.span.clone(),
                ));
            }
            lowered.push((expected.index(), value));
        }
        if seen.len() != layout.len() {
            let missing = layout
                .iter()
                .filter(|field| !seen.contains(&field.index()))
                .map(FieldDefinition::name)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(ExpressionError::new(
                "EXPRESSION_MISSING_NOMINAL_FIELD",
                format!("nominal construction is missing fields: {missing}"),
                expression.span.clone(),
            ));
        }
        Ok(lowered)
    }
}
