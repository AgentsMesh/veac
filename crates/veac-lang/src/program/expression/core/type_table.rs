use super::CoreTypeId;
use crate::program::expression::ValueType;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CoreType {
    Value(ValueType),
    MapBuilder { map_type: CoreTypeId },
    MapPending { map_type: CoreTypeId },
}

impl CoreType {
    pub fn value_type(&self) -> Option<&ValueType> {
        match self {
            Self::Value(value) => Some(value),
            Self::MapBuilder { .. } | Self::MapPending { .. } => None,
        }
    }

    pub fn is_internal(&self) -> bool {
        !matches!(self, Self::Value(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreTypeEntry {
    pub(crate) id: CoreTypeId,
    pub(crate) kind: CoreType,
}

impl CoreTypeEntry {
    pub fn id(&self) -> CoreTypeId {
        self.id
    }

    pub fn kind(&self) -> &CoreType {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreTypeTable {
    pub(crate) entries: Vec<CoreTypeEntry>,
}

impl CoreTypeTable {
    pub(crate) fn new(entries: Vec<CoreTypeEntry>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[CoreTypeEntry] {
        &self.entries
    }

    pub fn get(&self, id: CoreTypeId) -> Option<&CoreType> {
        id.index()
            .and_then(|index| self.entries.get(index))
            .map(CoreTypeEntry::kind)
    }

    pub fn value(&self, id: CoreTypeId) -> Option<&ValueType> {
        self.get(id).and_then(CoreType::value_type)
    }
}
