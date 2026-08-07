use crate::program::expression::{CoreTypeId, Value};

pub(super) enum MapState {
    Builder(MapBuilder),
    Pending(MapPending),
}

pub(super) struct MapBuilder {
    pub(super) map_type: CoreTypeId,
    pub(super) expected: u32,
    pub(super) next: u32,
    pub(super) entries: Vec<(Value, Value)>,
    pub(super) keys: Vec<Value>,
}

pub(super) struct MapPending {
    pub(super) builder: MapBuilder,
    pub(super) key: Value,
}
