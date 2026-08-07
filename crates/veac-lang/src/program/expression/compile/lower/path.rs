use super::Lowerer;
use crate::program::expression::ast::{static_path, Expression, PathSegment};
use crate::program::expression::hir::{TypedNode, TypedNodeKind, TypedStructProject};
use crate::program::expression::{ExpressionError, ValueType, ValueTypeKind};
use crate::program::TypeDefinitionKind;

impl Lowerer<'_> {
    pub(super) fn field_project(
        &mut self,
        receiver: &Expression,
        field: &str,
        field_span: &std::ops::Range<usize>,
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        if let Some(path) = static_path(expression) {
            if !self.shadows_static(&path[0].name) {
                if let Some((node, consumed)) = self.longest_static_value(&path)? {
                    return self.project_remaining(node, &path[consumed..], expression);
                }
                if let Some(variant) = self.unit_variant(&path, expression)? {
                    return Ok(variant);
                }
            }
        }
        let receiver = self.lower(receiver)?;
        self.project_field(receiver, field, field_span, expression)
    }

    fn longest_static_value(
        &mut self,
        path: &[PathSegment],
    ) -> Result<Option<(TypedNode, usize)>, ExpressionError> {
        for count in (1..=path.len()).rev() {
            let name = crate::program::expression::ast::join_path(&path[..count]);
            if (self.symbols)(&name).is_none() && !self.has_static_symbol(&name) {
                continue;
            }
            let expression = Expression {
                kind: crate::program::expression::ast::ExpressionKind::Symbol(name.clone()),
                span: path[0].span.start..path[count - 1].span.end,
            };
            let (kind, value_type) = self.symbol(&name, &expression)?;
            return Ok(Some((
                TypedNode {
                    kind,
                    value_type,
                    span: expression.span,
                },
                count,
            )));
        }
        Ok(None)
    }

    fn project_remaining(
        &mut self,
        mut receiver: TypedNode,
        fields: &[PathSegment],
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        for field in fields {
            receiver = self.projected_node(receiver, &field.name, &field.span, expression)?;
        }
        Ok((receiver.kind, receiver.value_type))
    }

    fn project_field(
        &mut self,
        receiver: TypedNode,
        field: &str,
        field_span: &std::ops::Range<usize>,
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let node = self.projected_node(receiver, field, field_span, expression)?;
        Ok((node.kind, node.value_type))
    }

    pub(super) fn projected_node(
        &self,
        receiver: TypedNode,
        field: &str,
        field_span: &std::ops::Range<usize>,
        _expression: &Expression,
    ) -> Result<TypedNode, ExpressionError> {
        let ValueTypeKind::Nominal(reference) = receiver.value_type.kind() else {
            return Err(field_type(field, &receiver.value_type, field_span));
        };
        let definition = self.types.definition(reference.id()).ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_NOMINAL_UNKNOWN_TYPE",
                format!("nominal receiver type `{reference}` is outside the verified registry"),
                receiver.span.clone(),
            )
        })?;
        let TypeDefinitionKind::Struct(layout) = definition.kind() else {
            return Err(field_type(field, &receiver.value_type, field_span));
        };
        let definition = layout.field(field).ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_UNKNOWN_FIELD",
                format!("struct `{reference}` has no field `{field}`"),
                field_span.clone(),
            )
        })?;
        let span = receiver.span.start..field_span.end;
        Ok(TypedNode {
            kind: TypedNodeKind::StructProject(TypedStructProject {
                receiver: Box::new(receiver),
                field: definition.index(),
            }),
            value_type: definition.value_type().clone(),
            span,
        })
    }
}

fn field_type(field: &str, receiver: &ValueType, span: &std::ops::Range<usize>) -> ExpressionError {
    ExpressionError::new(
        "EXPRESSION_FIELD_TYPE",
        format!("cannot project field `{field}` from {receiver}"),
        span.clone(),
    )
}
