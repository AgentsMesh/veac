use super::Lowerer;
use crate::program::expression::ast::{Expression, NominalField, PathSegment};
use crate::program::expression::hir::{TypedEnumConstruct, TypedNodeKind, TypedStructConstruct};
use crate::program::expression::{ExpressionError, ValueType};
use crate::program::TypeDefinitionKind;

mod fields;
mod resolve;
pub(in crate::program::expression::compile::lower) use resolve::ResolvedType;

impl Lowerer<'_> {
    pub(super) fn nominal_construct(
        &mut self,
        path: &[PathSegment],
        fields: &[NominalField],
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        self.require_static_type(path)?;
        let resolved = self.resolve_nominal(path, expression)?;
        let value_type = ValueType::nominal(resolved.reference.clone());
        match resolved.kind {
            TypeDefinitionKind::Struct(layout) if resolved.consumed == path.len() => {
                let fields = self.lower_nominal_fields(fields, layout.fields(), expression)?;
                Ok((
                    TypedNodeKind::StructConstruct(TypedStructConstruct {
                        nominal: resolved.reference.id(),
                        fields,
                    }),
                    value_type,
                ))
            }
            TypeDefinitionKind::Enum(layout) if resolved.consumed + 1 == path.len() => {
                let name = &path[resolved.consumed];
                let variant = layout.variant(&name.name).ok_or_else(|| {
                    ExpressionError::new(
                        "EXPRESSION_UNKNOWN_VARIANT",
                        format!("unknown enum variant `{}`", name.name),
                        name.span.clone(),
                    )
                })?;
                let fields = self.lower_nominal_fields(fields, variant.fields(), expression)?;
                Ok((
                    TypedNodeKind::EnumConstruct(TypedEnumConstruct {
                        nominal: resolved.reference.id(),
                        variant: variant.index(),
                        fields,
                    }),
                    value_type,
                ))
            }
            TypeDefinitionKind::Struct(_) => Err(kind_error(
                "struct construction cannot name a variant",
                expression,
            )),
            TypeDefinitionKind::Enum(_) => Err(kind_error(
                "enum construction requires exactly one variant segment",
                expression,
            )),
        }
    }

    pub(super) fn unit_variant(
        &mut self,
        path: &[PathSegment],
        expression: &Expression,
    ) -> Result<Option<(TypedNodeKind, ValueType)>, ExpressionError> {
        let Some(resolved) = self.resolve_nominal_prefix(path) else {
            return Ok(None);
        };
        let TypeDefinitionKind::Enum(layout) = resolved.kind else {
            return Ok(None);
        };
        if resolved.consumed + 1 != path.len() {
            return Ok(None);
        }
        let name = &path[resolved.consumed];
        let variant = layout.variant(&name.name).ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_UNKNOWN_VARIANT",
                format!("unknown enum variant `{}`", name.name),
                name.span.clone(),
            )
        })?;
        if !variant.fields().is_empty() {
            return Err(ExpressionError::new(
                "EXPRESSION_ENUM_PAYLOAD",
                format!(
                    "enum variant `{}` requires a payload construction",
                    name.name
                ),
                expression.span.clone(),
            ));
        }
        Ok(Some((
            TypedNodeKind::EnumConstruct(TypedEnumConstruct {
                nominal: resolved.reference.id(),
                variant: variant.index(),
                fields: Vec::new(),
            }),
            ValueType::nominal(resolved.reference),
        )))
    }

    fn require_static_type(&self, path: &[PathSegment]) -> Result<(), ExpressionError> {
        if self.shadows_static(&path[0].name) {
            Err(ExpressionError::new(
                "EXPRESSION_NOMINAL_SHADOWED",
                format!(
                    "value `{}` shadows the nominal type namespace",
                    path[0].name
                ),
                path[0].span.clone(),
            ))
        } else {
            Ok(())
        }
    }

    fn resolve_nominal(
        &self,
        path: &[PathSegment],
        expression: &Expression,
    ) -> Result<ResolvedType, ExpressionError> {
        self.resolve_nominal_prefix(path).ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_UNKNOWN_TYPE",
                format!(
                    "unknown nominal type in `{}`",
                    crate::program::expression::ast::join_path(path)
                ),
                expression.span.clone(),
            )
        })
    }
}

fn kind_error(message: &'static str, expression: &Expression) -> ExpressionError {
    ExpressionError::new("EXPRESSION_NOMINAL_KIND", message, expression.span.clone())
}
