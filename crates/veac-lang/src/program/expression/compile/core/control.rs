use super::Builder;
use crate::program::expression::ast::BinaryOperator;
use crate::program::expression::core::{BlockId, CoreBlockParameter, CoreTerminator, ValueId};
use crate::program::expression::hir::{TypedBlock, TypedNode, TypedStatement};
use crate::program::expression::{PrimitiveType, ValueType, ValueTypeKind};

impl Builder<'_> {
    pub(super) fn typed_block(&mut self, block: &TypedBlock) -> ValueId {
        for statement in &block.statements {
            match statement {
                TypedStatement::Let(binding) => {
                    let value = self.node(&binding.value);
                    self.locals.insert(binding.id, value);
                }
                TypedStatement::Var(binding) => self.local_init(binding),
                TypedStatement::Set(assignment) => self.local_set(assignment),
            }
        }
        self.node(&block.result)
    }

    pub(super) fn conditional(
        &mut self,
        condition: &TypedNode,
        then_branch: &TypedBlock,
        else_branch: &TypedBlock,
        node: &TypedNode,
    ) -> ValueId {
        let condition = self.node(condition);
        let then_target = self.new_block();
        let else_target = self.new_block();
        let join = self.new_block();
        self.terminate(CoreTerminator::Branch {
            condition,
            then_target,
            else_target,
            span: node.span.clone(),
        });
        self.current = then_target;
        let then_value = self.typed_block(then_branch);
        self.jump(join, then_value, then_branch.result.span.clone());
        self.current = else_target;
        let else_value = self.typed_block(else_branch);
        self.jump(join, else_value, else_branch.result.span.clone());
        let condition_metadata = self.metadata(condition);
        let then_metadata = self.metadata(then_value);
        let else_metadata = self.metadata(else_value);
        let mut metadata = crate::program::expression::core::CoreValueMetadata::combine([
            &condition_metadata,
            &then_metadata,
            &else_metadata,
        ]);
        if matches!(
            node.value_type.kind(),
            ValueTypeKind::Nominal(_) | ValueTypeKind::Function { .. }
        ) {
            metadata.join_contract_from(&[&then_metadata, &else_metadata]);
        }
        let result = self.join_parameter(join, &node.value_type, node.span.clone(), metadata);
        self.current = join;
        result
    }

    pub(super) fn logical(
        &mut self,
        operator: BinaryOperator,
        left: &TypedNode,
        right: &TypedNode,
        node: &TypedNode,
    ) -> ValueId {
        let left = self.node(left);
        let right_target = self.new_block();
        let short_target = self.new_block();
        let join = self.new_block();
        let (then_target, else_target) = match operator {
            BinaryOperator::LogicalAnd => (right_target, short_target),
            BinaryOperator::LogicalOr => (short_target, right_target),
            _ => unreachable!("only logical operators branch"),
        };
        self.terminate(CoreTerminator::Branch {
            condition: left,
            then_target,
            else_target,
            span: node.span.clone(),
        });
        self.current = right_target;
        let right_span = right.span.clone();
        let right = self.node(right);
        self.jump(join, right, right_span);
        self.current = short_target;
        self.jump(join, left, node.span.clone());
        let left_metadata = self.metadata(left);
        let right_metadata = self.metadata(right);
        let metadata = crate::program::expression::core::CoreValueMetadata::combine([
            &left_metadata,
            &right_metadata,
        ]);
        let boolean = ValueType::primitive(PrimitiveType::Boolean);
        let result = self.join_parameter(join, &boolean, node.span.clone(), metadata);
        self.current = join;
        result
    }

    pub(super) fn terminate(&mut self, terminator: CoreTerminator) {
        let slot = &mut self.current_mut().terminator;
        assert!(slot.replace(terminator).is_none(), "block terminates once");
    }

    pub(super) fn jump(&mut self, target: BlockId, value: ValueId, span: std::ops::Range<usize>) {
        self.terminate(CoreTerminator::Jump {
            target,
            arguments: vec![value],
            span,
        });
    }

    pub(super) fn new_block(&mut self) -> BlockId {
        let id = BlockId::new(u32::try_from(self.blocks.len()).expect("block limit fits u32"));
        self.blocks.push(super::PendingBlock {
            id,
            parameters: Vec::new(),
            instructions: Vec::new(),
            terminator: None,
        });
        id
    }

    pub(super) fn join_parameter(
        &mut self,
        block: BlockId,
        value_type: &ValueType,
        span: std::ops::Range<usize>,
        metadata: crate::program::expression::core::CoreValueMetadata,
    ) -> ValueId {
        let id = self.value_id();
        let type_id = self.types.intern_value(value_type);
        self.blocks[block.index().expect("block ID fits usize")]
            .parameters
            .push(CoreBlockParameter {
                id,
                type_id,
                metadata: metadata.clone(),
                span,
            });
        self.metadata.insert(id, metadata);
        id
    }
}
