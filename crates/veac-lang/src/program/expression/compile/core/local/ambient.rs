use super::Builder;
use crate::program::expression::core::{CoreValueMetadata, Stage};
use crate::program::expression::hir::{TypedBlock, TypedNode, TypedNodeKind, TypedStatement};
use crate::program::expression::ValueType;

impl Builder<'_> {
    pub(in crate::program::expression::compile::core) fn prepare(
        &mut self,
        root: &TypedNode,
        parameters: &[ValueType],
        parameter_stages: &[Stage],
        captures: &[ValueType],
    ) {
        debug_assert_eq!(parameters.len(), parameter_stages.len());
        self.parameter_stages = parameter_stages.to_vec();
        self.prepare_node(root);
        for (index, (value_type, stage)) in parameters.iter().zip(parameter_stages).enumerate() {
            self.absorb_ambient(CoreValueMetadata::typed_parameter(
                *stage, index, value_type,
            ));
        }
        for (index, value_type) in captures.iter().enumerate() {
            self.absorb_ambient(CoreValueMetadata::capture(Stage::Const, index, value_type));
        }
    }

    pub(in crate::program::expression::compile::core) fn prepare_block(
        &mut self,
        block: &TypedBlock,
        parameters: &[ValueType],
        parameter_stages: &[Stage],
        captures: &[ValueType],
    ) {
        self.prepare_statements(block);
        self.prepare(&block.result, parameters, parameter_stages, captures);
    }

    fn prepare_node(&mut self, node: &TypedNode) {
        match &node.kind {
            TypedNodeKind::External(name) => {
                let id = self.input(name, node);
                self.absorb_ambient(self.inputs[id.index().expect("dense input ID")].metadata());
            }
            TypedNodeKind::Literal(_)
            | TypedNodeKind::Parameter(_)
            | TypedNodeKind::Local(_)
            | TypedNodeKind::MutableLocal(_)
            | TypedNodeKind::Capture(_) => {}
            TypedNodeKind::Unary { operand, .. } => self.prepare_node(operand),
            TypedNodeKind::Binary { left, right, .. } => self.prepare_nodes([left, right]),
            TypedNodeKind::Range { start, end, step } => {
                self.prepare_nodes([start, end]);
                if let Some(step) = step {
                    self.prepare_node(step);
                }
            }
            TypedNodeKind::Closure { captures, .. } => {
                for capture in captures {
                    self.prepare_node(&capture.source);
                }
            }
            TypedNodeKind::Call { arguments, .. } => self.prepare_arguments(arguments),
            TypedNodeKind::Collection { arguments, .. } => self.prepare_slice(arguments),
            TypedNodeKind::ForEach { iterable, body, .. } => {
                self.prepare_node(iterable);
                self.prepare_node(body);
            }
            TypedNodeKind::Invoke { callee, arguments } => {
                self.prepare_node(callee);
                self.prepare_slice(arguments);
            }
            TypedNodeKind::MethodCall(call) => {
                self.prepare_node(&call.receiver);
                self.prepare_arguments(&call.arguments);
            }
            TypedNodeKind::DomainCall(call) => self.prepare_arguments(&call.operands),
            TypedNodeKind::TemporalAttach(value) => {
                self.prepare_node(&value.owner);
                self.prepare_slice(&value.selectors);
                self.prepare_node(&value.animation);
            }
            TypedNodeKind::StructConstruct(value) => self.prepare_fields(&value.fields),
            TypedNodeKind::EnumConstruct(value) => self.prepare_fields(&value.fields),
            TypedNodeKind::StructProject(value) => self.prepare_node(&value.receiver),
            TypedNodeKind::Match { scrutinee, arms } => {
                self.prepare_node(scrutinee);
                for arm in arms {
                    self.prepare_statements(&arm.body);
                    self.prepare_node(&arm.body.result);
                }
            }
            TypedNodeKind::List(values) | TypedNodeKind::Tuple(values) => {
                self.prepare_slice(values)
            }
            TypedNodeKind::Map(entries) => {
                for entry in entries {
                    self.prepare_nodes([&entry.key, &entry.value]);
                }
            }
            TypedNodeKind::Block(block) => {
                self.prepare_statements(block);
                self.prepare_node(&block.result);
            }
            TypedNodeKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.prepare_node(condition);
                for branch in [then_branch, else_branch] {
                    self.prepare_statements(branch);
                    self.prepare_node(&branch.result);
                }
            }
        }
    }

    fn prepare_statements(&mut self, block: &TypedBlock) {
        for statement in &block.statements {
            let value = match statement {
                TypedStatement::Let(value) => &value.value,
                TypedStatement::Var(value) => &value.value,
                TypedStatement::Set(value) => &value.value,
            };
            self.prepare_node(value);
        }
    }

    fn prepare_slice(&mut self, values: &[TypedNode]) {
        for value in values {
            self.prepare_node(value);
        }
    }

    fn prepare_arguments(
        &mut self,
        arguments: &[crate::program::expression::hir::TypedCallArgument],
    ) {
        for argument in arguments {
            self.prepare_node(&argument.value);
        }
    }

    fn prepare_nodes<const N: usize>(&mut self, values: [&TypedNode; N]) {
        for value in values {
            self.prepare_node(value);
        }
    }

    fn prepare_fields(&mut self, fields: &[(crate::program::FieldIndex, TypedNode)]) {
        for (_, value) in fields {
            self.prepare_node(value);
        }
    }

    fn absorb_ambient(&mut self, value: CoreValueMetadata) {
        self.ambient = CoreValueMetadata::combine([&self.ambient, &value]);
    }
}
