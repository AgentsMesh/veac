use crate::program::expression::core::{CoreType, CoreTypeEntry, CoreTypeId, CoreTypeTable};
use crate::program::expression::{ValueType, ValueTypeKind};

#[derive(Default)]
pub(super) struct TypeTableBuilder {
    entries: Vec<CoreTypeEntry>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct MapTypeIds {
    pub(super) value: CoreTypeId,
    pub(super) builder: CoreTypeId,
}

impl TypeTableBuilder {
    pub(super) fn intern_value(&mut self, value: &ValueType) -> CoreTypeId {
        self.intern_children(value);
        if let Some(entry) = self
            .entries
            .iter()
            .find(|entry| matches!(&entry.kind, CoreType::Value(existing) if existing == value))
        {
            return entry.id;
        }
        let id = CoreTypeId::new(
            u32::try_from(self.entries.len()).expect("Core type limit fits in u32"),
        );
        self.entries.push(CoreTypeEntry {
            id,
            kind: CoreType::Value(value.clone()),
        });
        id
    }

    pub(super) fn finish(self) -> CoreTypeTable {
        CoreTypeTable::new(self.entries)
    }

    pub(super) fn intern_map_builder(&mut self, value: &ValueType) -> MapTypeIds {
        let value = self.intern_value(value);
        let builder = self.intern_core(CoreType::MapBuilder { map_type: value });
        MapTypeIds { value, builder }
    }

    pub(super) fn intern_map_pending(&mut self, map_type: CoreTypeId) -> CoreTypeId {
        self.intern_core(CoreType::MapPending { map_type })
    }

    fn intern_children(&mut self, value: &ValueType) {
        match value.kind() {
            ValueTypeKind::Primitive(_) | ValueTypeKind::Domain(_) | ValueTypeKind::Nominal(_) => {}
            ValueTypeKind::List(element) | ValueTypeKind::Range(element) => {
                self.intern_value(element);
            }
            ValueTypeKind::Map { key, value } => {
                self.intern_value(&ValueType::primitive(key.primitive()));
                self.intern_value(value);
            }
            ValueTypeKind::Tuple(elements) => {
                for element in elements {
                    self.intern_value(element);
                }
            }
            ValueTypeKind::Function {
                parameters, result, ..
            } => {
                for parameter in parameters {
                    self.intern_value(parameter);
                }
                self.intern_value(result);
            }
        }
    }

    fn intern_core(&mut self, kind: CoreType) -> CoreTypeId {
        if let Some(entry) = self.entries.iter().find(|entry| entry.kind == kind) {
            return entry.id;
        }
        let id = CoreTypeId::new(
            u32::try_from(self.entries.len()).expect("Core type limit fits in u32"),
        );
        self.entries.push(CoreTypeEntry { id, kind });
        id
    }
}
