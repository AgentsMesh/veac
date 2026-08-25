use std::mem::{size_of, size_of_val};

use super::{TypedBlock, TypedExpression, TypedNode, TypedNodeKind};

impl TypedExpression {
    pub(crate) fn cache_tree_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(node(&self.root))
    }

    pub(crate) fn cache_shared_bytes(&self) -> usize {
        size_of_val(self.types.as_ref())
            .saturating_add(self.types.retained_bytes())
            .saturating_add(size_of_val(self.domain.as_ref()))
            .saturating_add(self.domain.retained_bytes())
    }
}

fn node(value: &TypedNode) -> usize {
    let mut bytes = size_of_val(value).saturating_add(type_bytes(&value.value_type));
    bytes = bytes.saturating_add(match &value.kind {
        TypedNodeKind::Literal(value) => value
            .retained_bytes()
            .saturating_add(type_bytes(&value.value_type())),
        TypedNodeKind::External(name) => name.len(),
        TypedNodeKind::Parameter(_)
        | TypedNodeKind::Local(_)
        | TypedNodeKind::MutableLocal(_)
        | TypedNodeKind::Capture(_) => 0,
        TypedNodeKind::Unary { operand, .. } => node(operand),
        TypedNodeKind::Binary { left, right, .. } => node(left).saturating_add(node(right)),
        TypedNodeKind::Range { start, end, step } => node(start)
            .saturating_add(node(end))
            .saturating_add(step.as_deref().map_or(0, node)),
        TypedNodeKind::Closure {
            parameters,
            captures,
            body,
            ..
        } => parameters
            .iter()
            .map(|value| size_of_val(value).saturating_add(type_bytes(&value.value_type)))
            .sum::<usize>()
            .saturating_add(captures.iter().map(capture).sum())
            .saturating_add(block(body)),
        TypedNodeKind::Call {
            arguments,
            defaults,
            ..
        } => arguments
            .iter()
            .map(call_argument)
            .sum::<usize>()
            .saturating_add(defaults.iter().map(default_argument).sum()),
        TypedNodeKind::Invoke { callee, arguments } => {
            node(callee).saturating_add(arguments.iter().map(node).sum())
        }
        TypedNodeKind::MethodCall(value) => node(&value.receiver)
            .saturating_add(value.arguments.iter().map(call_argument).sum::<usize>())
            .saturating_add(value.defaults.iter().map(default_argument).sum::<usize>()),
        TypedNodeKind::DomainCall(value) => value.operands.iter().map(call_argument).sum(),
        TypedNodeKind::TemporalAttach(value) => node(&value.owner)
            .saturating_add(value.selectors.iter().map(node).sum::<usize>())
            .saturating_add(node(&value.animation)),
        TypedNodeKind::StructConstruct(value) => value.fields.iter().map(nominal_field).sum(),
        TypedNodeKind::EnumConstruct(value) => value.fields.iter().map(nominal_field).sum(),
        TypedNodeKind::StructProject(value) => node(&value.receiver),
        TypedNodeKind::Match { scrutinee, arms } => {
            node(scrutinee).saturating_add(arms.iter().map(match_arm).sum::<usize>())
        }
        TypedNodeKind::Collection { arguments, .. } => arguments.iter().map(node).sum(),
        TypedNodeKind::ForEach { iterable, body, .. } => node(iterable).saturating_add(node(body)),
        TypedNodeKind::List(values) | TypedNodeKind::Tuple(values) => values.iter().map(node).sum(),
        TypedNodeKind::Map(values) => values.iter().map(map_entry).sum(),
        TypedNodeKind::Block(value) => block(value),
        TypedNodeKind::If {
            condition,
            then_branch,
            else_branch,
        } => node(condition)
            .saturating_add(block(then_branch))
            .saturating_add(block(else_branch)),
    });
    bytes
}

fn block(value: &TypedBlock) -> usize {
    size_of_val(value)
        .saturating_add(value.statements.iter().map(statement).sum::<usize>())
        .saturating_add(node(&value.result))
}

fn statement(value: &super::TypedStatement) -> usize {
    let child = match value {
        super::TypedStatement::Let(value) => &value.value,
        super::TypedStatement::Var(value) => &value.value,
        super::TypedStatement::Set(value) => &value.value,
    };
    size_of_val(value)
        .saturating_sub(size_of::<TypedNode>())
        .saturating_add(node(child))
}

fn call_argument(value: &super::TypedCallArgument) -> usize {
    size_of_val(value)
        .saturating_sub(size_of::<TypedNode>())
        .saturating_add(node(&value.value))
}

fn default_argument(value: &super::TypedDefaultArgument) -> usize {
    size_of_val(value).saturating_add(type_bytes(&value.value_type))
}

fn capture(value: &super::TypedCapture) -> usize {
    size_of_val(value)
        .saturating_sub(size_of::<TypedNode>())
        .saturating_add(node(&value.source))
}

fn nominal_field(value: &(crate::program::FieldIndex, TypedNode)) -> usize {
    size_of_val(value)
        .saturating_sub(size_of::<TypedNode>())
        .saturating_add(node(&value.1))
}

fn match_arm(value: &super::TypedMatchArm) -> usize {
    size_of_val(value)
        .saturating_add(
            value
                .bindings
                .len()
                .saturating_mul(size_of::<super::TypedPatternBinding>()),
        )
        .saturating_add(block(&value.body))
}

fn map_entry(value: &super::TypedMapEntry) -> usize {
    size_of_val(value)
        .saturating_sub(size_of::<TypedNode>().saturating_mul(2))
        .saturating_add(node(&value.key))
        .saturating_add(node(&value.value))
}

fn type_bytes(value: &super::super::ValueType) -> usize {
    super::super::value_type::retained_shape_bytes(value).unwrap_or(usize::MAX)
}
