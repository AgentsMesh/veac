use super::Builder;
use crate::program::expression::core::{CoreInstructionKind, CoreValueMetadata, ValueId};
use crate::program::expression::hir::{TypedMapEntry, TypedNode};

impl Builder<'_> {
    pub(super) fn list(&mut self, values: &[TypedNode], node: &TypedNode) -> ValueId {
        let elements = values.iter().map(|value| self.node(value)).collect();
        self.emit(
            CoreInstructionKind::List { elements },
            &node.value_type,
            node.span.clone(),
        )
    }

    pub(super) fn tuple(&mut self, values: &[TypedNode], node: &TypedNode) -> ValueId {
        let elements = values.iter().map(|value| self.node(value)).collect();
        self.emit(
            CoreInstructionKind::Tuple { elements },
            &node.value_type,
            node.span.clone(),
        )
    }

    pub(super) fn map(&mut self, entries: &[TypedMapEntry], node: &TypedNode) -> ValueId {
        let types = self.types.intern_map_builder(&node.value_type);
        let count = u32::try_from(entries.len()).expect("expression node limit fits u32");
        let mut builder = self.emit_core(
            CoreInstructionKind::MapBegin { entries: count },
            types.builder,
            CoreValueMetadata::constant(),
            node.span.clone(),
        );
        let mut pending_type = None;
        for (ordinal, entry) in entries.iter().enumerate() {
            let key = self.node(&entry.key);
            let pending_type =
                *pending_type.get_or_insert_with(|| self.types.intern_map_pending(types.value));
            let kind = CoreInstructionKind::MapKey {
                builder,
                key,
                ordinal: u32::try_from(ordinal).expect("expression node limit fits u32"),
            };
            let metadata = self.instruction_metadata(&kind, &node.value_type);
            let pending = self.emit_core(kind, pending_type, metadata, entry.key.span.clone());
            let value = self.node(&entry.value);
            let kind = CoreInstructionKind::MapValue { pending, value };
            let metadata = self.instruction_metadata(&kind, &node.value_type);
            builder = self.emit_core(kind, types.builder, metadata, entry.value.span.clone());
        }
        let kind = CoreInstructionKind::MapFinish { builder };
        let metadata = self.instruction_metadata(&kind, &node.value_type);
        self.emit_core(kind, types.value, metadata, node.span.clone())
    }
}
