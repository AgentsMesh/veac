use std::collections::BTreeSet;

use super::{Lowerer, ResolvedArm, ResolvedBinding};
use crate::program::expression::ast::{MatchArm, MatchPattern, PatternField};
use crate::program::expression::ExpressionError;
use crate::program::{EnumDefinition, EnumVariantDefinition, TypeDefinitionKind, TypeRef};

impl Lowerer<'_> {
    pub(super) fn resolve_match_arms<'a>(
        &self,
        reference: &TypeRef,
        layout: &EnumDefinition,
        arms: &'a [MatchArm],
        expression: &crate::program::expression::ast::Expression,
    ) -> Result<Vec<ResolvedArm<'a>>, ExpressionError> {
        let mut resolved = Vec::new();
        let mut seen = BTreeSet::new();
        let mut wildcard = None;
        for arm in arms {
            match &arm.pattern {
                MatchPattern::Variant { path, fields } => {
                    let variant = self.resolve_variant(reference, layout, path)?;
                    if !seen.insert(variant.index()) {
                        return Err(ExpressionError::new(
                            "EXPRESSION_DUPLICATE_MATCH_VARIANT",
                            format!("variant `{}` has more than one match arm", variant.name()),
                            arm.span.clone(),
                        ));
                    }
                    resolved.push(ResolvedArm {
                        variant: variant.index(),
                        bindings: pattern_bindings(variant, fields, &arm.span)?,
                        body: &arm.body,
                        span: arm.span.clone(),
                    });
                }
                MatchPattern::Wildcard { span } => wildcard = Some((&arm.body, span, &arm.span)),
            }
        }
        if let Some((body, span, arm_span)) = wildcard {
            let remaining = layout
                .variants()
                .iter()
                .filter(|variant| !seen.contains(&variant.index()))
                .collect::<Vec<_>>();
            if remaining.is_empty() {
                return Err(ExpressionError::new(
                    "EXPRESSION_UNREACHABLE_WILDCARD",
                    "wildcard arm cannot match any remaining variant",
                    span.clone(),
                ));
            }
            resolved.extend(remaining.into_iter().map(|variant| ResolvedArm {
                variant: variant.index(),
                bindings: Vec::new(),
                body,
                span: arm_span.clone(),
            }));
        } else if seen.len() != layout.variants().len() {
            let missing = layout
                .variants()
                .iter()
                .filter(|variant| !seen.contains(&variant.index()))
                .map(EnumVariantDefinition::name)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(ExpressionError::new(
                "EXPRESSION_NON_EXHAUSTIVE_MATCH",
                format!("match is missing variants: {missing}"),
                expression.span.clone(),
            ));
        }
        resolved.sort_by_key(|arm| arm.variant);
        Ok(resolved)
    }

    fn resolve_variant<'a>(
        &self,
        expected: &TypeRef,
        layout: &'a EnumDefinition,
        path: &[crate::program::expression::ast::PathSegment],
    ) -> Result<&'a EnumVariantDefinition, ExpressionError> {
        let resolved = self.resolve_nominal_prefix(path).ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_UNKNOWN_TYPE",
                "match pattern names an unknown enum type",
                path_span(path),
            )
        })?;
        if resolved.reference.id() != expected.id() {
            return Err(ExpressionError::new(
                "EXPRESSION_MATCH_PATTERN_TYPE",
                format!(
                    "pattern type {} does not match scrutinee type {expected}",
                    resolved.reference
                ),
                path_span(path),
            ));
        }
        if !matches!(resolved.kind, TypeDefinitionKind::Enum(_))
            || resolved.consumed + 1 != path.len()
        {
            return Err(ExpressionError::new(
                "EXPRESSION_MATCH_PATTERN",
                "match pattern requires exactly one enum variant segment",
                path_span(path),
            ));
        }
        let variant = &path[resolved.consumed];
        layout.variant(&variant.name).ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_UNKNOWN_VARIANT",
                format!("unknown enum variant `{}`", variant.name),
                variant.span.clone(),
            )
        })
    }
}

fn pattern_bindings(
    variant: &EnumVariantDefinition,
    fields: &[PatternField],
    pattern_span: &std::ops::Range<usize>,
) -> Result<Vec<ResolvedBinding>, ExpressionError> {
    let mut names = BTreeSet::new();
    let mut bindings = BTreeSet::new();
    let resolved = fields
        .iter()
        .map(|field| {
            let definition = variant.field(&field.field).ok_or_else(|| {
                ExpressionError::new(
                    "EXPRESSION_UNKNOWN_PATTERN_FIELD",
                    format!(
                        "variant `{}` has no field `{}`",
                        variant.name(),
                        field.field
                    ),
                    field.field_span.clone(),
                )
            })?;
            if !names.insert(definition.index()) || !bindings.insert(&field.binding) {
                return Err(ExpressionError::new(
                    "EXPRESSION_DUPLICATE_PATTERN_NAME",
                    "pattern field and binding names must be unique",
                    field.binding_span.clone(),
                ));
            }
            Ok(ResolvedBinding {
                name: field.binding.clone(),
                field: definition.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if names.len() != variant.fields().len() {
        let missing = variant
            .fields()
            .iter()
            .filter(|field| !names.contains(&field.index()))
            .map(|field| field.name())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(ExpressionError::new(
            "EXPRESSION_INCOMPLETE_PATTERN",
            format!(
                "variant `{}` pattern is missing fields: {missing}",
                variant.name()
            ),
            pattern_span.clone(),
        ));
    }
    Ok(resolved)
}

fn path_span(path: &[crate::program::expression::ast::PathSegment]) -> std::ops::Range<usize> {
    path[0].span.start..path[path.len() - 1].span.end
}
