use std::collections::BTreeSet;

use super::{compiled, instruction, slots, Evaluator};
use crate::program::expression::core::{CoreInstructionKind, CoreType, CoreTypeId, ValueId};
use crate::program::expression::{Environment, ExecutionBudget, MapValueEntry, Value};
use crate::program::DomainOperationRegistry;

use super::super::super::slot::{MapBuilderState, MapPendingState, RuntimeValue};

#[test]
fn map_runtime_protocol_rejects_invalid_tokens_and_payloads() {
    let map = compiled("#{\"key\": 1}");
    let begin = instruction(map.core(), |kind| {
        matches!(kind, CoreInstructionKind::MapBegin { .. })
    });
    let key = instruction(map.core(), |kind| {
        matches!(kind, CoreInstructionKind::MapKey { .. })
    });
    let value = instruction(map.core(), |kind| {
        matches!(kind, CoreInstructionKind::MapValue { .. })
    });
    let finish = instruction(map.core(), |kind| {
        matches!(kind, CoreInstructionKind::MapFinish { .. })
    });
    let Some(CoreType::MapBuilder { map_type }) = map.core().types.get(begin.type_id) else {
        unreachable!()
    };
    let environment = Environment::new();
    let execution = ExecutionBudget::default();
    let domain = DomainOperationRegistry::standard();
    let evaluator = Evaluator::new(&environment, &execution, map.registry_arc(), &domain);
    let builder = evaluator.map_begin(1, &begin, map.core()).unwrap();
    let mut invalid_ordinal = slots([
        (ValueId::new(0), builder),
        (
            ValueId::new(1),
            RuntimeValue::Public(Value::Text("key".into())),
        ),
    ]);
    assert!(evaluator
        .map_key(
            ValueId::new(0),
            ValueId::new(1),
            1,
            &key,
            &mut invalid_ordinal
        )
        .is_err());

    let mut wrong_key = slots([
        (
            ValueId::new(0),
            evaluator.map_begin(1, &begin, map.core()).unwrap(),
        ),
        (ValueId::new(1), RuntimeValue::Public(Value::Bool(true))),
    ]);
    assert!(evaluator
        .map_key(ValueId::new(0), ValueId::new(1), 0, &key, &mut wrong_key)
        .is_err());
    let pending = MapBuilderState {
        map_type: *map_type,
        expected: 1,
        next: 1,
        entries: Vec::new(),
        keys: BTreeSet::new(),
    };
    let mut wrong_value = slots([
        (
            ValueId::new(0),
            RuntimeValue::MapPending(MapPendingState {
                builder: pending,
                key: Value::Text("key".into()),
            }),
        ),
        (ValueId::new(1), RuntimeValue::Public(Value::Bool(true))),
    ]);
    assert!(evaluator
        .map_value(
            ValueId::new(0),
            ValueId::new(1),
            &value,
            map.core(),
            &mut wrong_value
        )
        .is_err());
    let invalid = MapBuilderState {
        map_type: CoreTypeId::new(u32::MAX),
        expected: 1,
        next: 1,
        entries: vec![MapValueEntry::new(
            Value::Text("key".into()),
            Value::Integer(1),
        )],
        keys: BTreeSet::new(),
    };
    let mut invalid = slots([(ValueId::new(0), RuntimeValue::MapBuilder(invalid))]);
    assert!(evaluator
        .map_finish(ValueId::new(0), &finish, map.core(), &mut invalid)
        .is_err());
}
