use super::CallSite;
use crate::program::expression::hir::{
    CallTarget, TypedBlock, TypedExpression, TypedNode, TypedNodeKind,
};

pub(super) fn calls(expression: &TypedExpression) -> Vec<CallSite> {
    let mut output = Vec::new();
    node(&expression.root, &mut output);
    output
}

fn node(value: &TypedNode, output: &mut Vec<CallSite>) {
    match &value.kind {
        TypedNodeKind::Literal(_)
        | TypedNodeKind::External(_)
        | TypedNodeKind::Parameter(_)
        | TypedNodeKind::Local(_)
        | TypedNodeKind::MutableLocal(_)
        | TypedNodeKind::Capture(_) => {}
        TypedNodeKind::Unary { operand, .. } => node(operand, output),
        TypedNodeKind::Binary { left, right, .. } => {
            node(left, output);
            node(right, output);
        }
        TypedNodeKind::Range { start, end, step } => {
            node(start, output);
            node(end, output);
            if let Some(step) = step {
                node(step, output);
            }
        }
        TypedNodeKind::Closure { captures, body, .. } => {
            for capture in captures {
                node(&capture.source, output);
            }
            block(body, output);
        }
        TypedNodeKind::Call { target, arguments } => {
            if let CallTarget::User(target) = target {
                output.push(CallSite {
                    target: *target,
                    span: value.span.clone(),
                });
            }
            nodes(arguments, output);
        }
        TypedNodeKind::Invoke { callee, arguments } => {
            node(callee, output);
            nodes(arguments, output);
        }
        TypedNodeKind::MethodCall(call) => {
            output.push(CallSite {
                target: call.target,
                span: value.span.clone(),
            });
            node(&call.receiver, output);
            nodes(&call.arguments, output);
        }
        TypedNodeKind::DomainCall(call) => nodes(&call.operands, output),
        TypedNodeKind::TemporalAttach(value) => {
            node(&value.owner, output);
            nodes(&value.selectors, output);
            node(&value.animation, output);
        }
        TypedNodeKind::StructConstruct(value) => fields(&value.fields, output),
        TypedNodeKind::EnumConstruct(value) => fields(&value.fields, output),
        TypedNodeKind::StructProject(value) => node(&value.receiver, output),
        TypedNodeKind::Match { scrutinee, arms } => {
            node(scrutinee, output);
            for arm in arms {
                block(&arm.body, output);
            }
        }
        TypedNodeKind::Collection { arguments, .. } => nodes(arguments, output),
        TypedNodeKind::ForEach { iterable, body, .. } => {
            node(iterable, output);
            node(body, output);
        }
        TypedNodeKind::List(values) | TypedNodeKind::Tuple(values) => nodes(values, output),
        TypedNodeKind::Map(entries) => {
            for entry in entries {
                node(&entry.key, output);
                node(&entry.value, output);
            }
        }
        TypedNodeKind::Block(value) => block(value, output),
        TypedNodeKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            node(condition, output);
            block(then_branch, output);
            block(else_branch, output);
        }
    }
}

fn block(value: &TypedBlock, output: &mut Vec<CallSite>) {
    for statement in &value.statements {
        use crate::program::expression::hir::TypedStatement;
        match statement {
            TypedStatement::Let(value) => node(&value.value, output),
            TypedStatement::Var(value) => node(&value.value, output),
            TypedStatement::Set(value) => node(&value.value, output),
        }
    }
    node(&value.result, output);
}

fn nodes(values: &[TypedNode], output: &mut Vec<CallSite>) {
    for value in values {
        node(value, output);
    }
}

fn fields(values: &[(crate::program::FieldIndex, TypedNode)], output: &mut Vec<CallSite>) {
    for (_, value) in values {
        node(value, output);
    }
}
