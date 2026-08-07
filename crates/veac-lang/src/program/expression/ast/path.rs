use super::{Expression, ExpressionKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PathSegment {
    pub name: String,
    pub span: std::ops::Range<usize>,
}

pub(crate) fn static_path(expression: &Expression) -> Option<Vec<PathSegment>> {
    match &expression.kind {
        ExpressionKind::Symbol(name) => Some(vec![PathSegment {
            name: name.clone(),
            span: expression.span.clone(),
        }]),
        ExpressionKind::FieldProject {
            receiver,
            field,
            field_span,
        } => {
            let mut path = static_path(receiver)?;
            path.push(PathSegment {
                name: field.clone(),
                span: field_span.clone(),
            });
            Some(path)
        }
        _ => None,
    }
}

pub(crate) fn join_path(path: &[PathSegment]) -> String {
    path.iter()
        .map(|segment| segment.name.as_str())
        .collect::<Vec<_>>()
        .join(".")
}
