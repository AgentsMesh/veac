use super::super::ast::BinaryOperator;
use super::super::core::{
    CoreCallTarget, CoreCallableInput, CoreInputIdentity, CoreInstructionKind, CoreProgram,
    FunctionRegistry, ValueId,
};
use super::super::hir::{CallTarget, TypedExpression, TypedNode, TypedNodeKind};

mod aggregate;
mod arguments;
mod builder;
mod call;
mod closure;
mod collection;
mod control;
mod domain;
mod finish;
mod for_each;
mod local;
mod matching;
mod model;
mod nominal;
mod nominal_value;
mod operator;
mod range;
mod temporal_attachment;
mod type_table;

use model::{Builder, PendingBlock};
use operator::{binary, unary};

pub(super) fn lower(
    expression: &TypedExpression,
    registry: &FunctionRegistry,
    trusted_functions: &dyn Fn(&str) -> bool,
    callable_inputs: &dyn Fn(&str) -> Option<CoreCallableInput>,
    input_identity: &dyn Fn(&str) -> CoreInputIdentity,
    extra_types: &[super::super::ValueType],
) -> CoreProgram {
    let mut builder = Builder::new(
        registry,
        trusted_functions,
        callable_inputs,
        input_identity,
        &expression.types,
        &expression.domain,
    );
    let parameter_stages = vec![super::super::core::Stage::Const; extra_types.len()];
    builder.prepare(&expression.root, extra_types, &parameter_stages, &[]);
    let result = builder.node(&expression.root);
    builder.finish(
        result,
        expression.result_type(),
        expression.root.span.clone(),
        extra_types,
    )
}

impl Builder<'_> {
    fn node(&mut self, node: &TypedNode) -> ValueId {
        let kind = match &node.kind {
            TypedNodeKind::Literal(value) => CoreInstructionKind::Literal(value.clone()),
            TypedNodeKind::External(name) => CoreInstructionKind::Input(self.input(name, node)),
            TypedNodeKind::Parameter(index) => CoreInstructionKind::Parameter(*index),
            TypedNodeKind::Capture(index) => CoreInstructionKind::Capture(*index),
            TypedNodeKind::Local(id) => {
                return *self.locals.get(id).expect("typed local must be bound")
            }
            TypedNodeKind::MutableLocal(id) => return self.local_get(*id, node),
            TypedNodeKind::Unary { operator, operand } => {
                let operand = self.node(operand);
                CoreInstructionKind::Unary {
                    operator: unary(*operator),
                    operand,
                }
            }
            TypedNodeKind::Binary {
                operator,
                left,
                right,
            } if matches!(
                operator,
                BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr
            ) =>
            {
                return self.logical(*operator, left, right, node)
            }
            TypedNodeKind::Binary {
                operator,
                left,
                right,
            } => {
                let left = self.node(left);
                let right = self.node(right);
                binary(*operator, left, right)
            }
            TypedNodeKind::Range { start, end, step } => {
                return self.range(start, end, step.as_deref(), node)
            }
            TypedNodeKind::Closure {
                parameters,
                captures,
                body,
                non_escaping,
            } => return self.closure(parameters, captures, body, *non_escaping, node),
            TypedNodeKind::Call {
                target,
                arguments,
                defaults,
            } => {
                let arguments = self.call_arguments(arguments, defaults, node.span.clone());
                let target = match target {
                    CallTarget::Builtin(function) => CoreCallTarget::Builtin(*function),
                    CallTarget::User(id) => CoreCallTarget::User(*id),
                };
                CoreInstructionKind::Call { target, arguments }
            }
            TypedNodeKind::Invoke { callee, arguments } => {
                let callee = self.node(callee);
                let arguments = arguments.iter().map(|value| self.node(value)).collect();
                CoreInstructionKind::Invoke { callee, arguments }
            }
            TypedNodeKind::MethodCall(call) => return self.method_call(call, node),
            TypedNodeKind::DomainCall(call) => return self.domain_call(call, node),
            TypedNodeKind::TemporalAttach(value) => return self.temporal_attachment(value, node),
            TypedNodeKind::StructConstruct(value) => return self.structure(value, node),
            TypedNodeKind::EnumConstruct(value) => return self.enumeration(value, node),
            TypedNodeKind::StructProject(value) => return self.project(value, node),
            TypedNodeKind::Match { scrutinee, arms } => {
                return self.match_value(scrutinee, arms, node)
            }
            TypedNodeKind::Collection {
                operation,
                arguments,
            } => return self.aggregate(*operation, arguments, node),
            TypedNodeKind::ForEach {
                iterable,
                body,
                iteration,
            } => return self.for_each(iterable, body, iteration, node),
            TypedNodeKind::List(values) => return self.list(values, node),
            TypedNodeKind::Map(entries) => return self.map(entries, node),
            TypedNodeKind::Tuple(values) => return self.tuple(values, node),
            TypedNodeKind::Block(block) => return self.typed_block(block),
            TypedNodeKind::If {
                condition,
                then_branch,
                else_branch,
            } => return self.conditional(condition, then_branch, else_branch, node),
        };
        self.emit(kind, &node.value_type, node.span.clone())
    }
}
