use super::super::{
    BuiltinFunction, CollectionOperation, PrimitiveType, UnitDimension, UnitSuffix,
};

#[test]
fn value_types_round_trip_every_canonical_spelling() {
    for kind in PrimitiveType::ALL {
        assert_eq!(PrimitiveType::parse(kind.as_str()), Some(kind));
    }
    assert_eq!(PrimitiveType::parse("unknown"), None);
}

#[test]
fn builtins_round_trip_every_canonical_spelling() {
    for function in BuiltinFunction::ALL {
        assert_eq!(BuiltinFunction::parse(function.as_str()), Some(function));
    }
    assert_eq!(BuiltinFunction::parse("unknown"), None);
}

#[test]
fn collection_operations_round_trip_every_closed_spelling() {
    assert_eq!(CollectionOperation::ALL.len(), 3);
    for operation in CollectionOperation::ALL {
        assert_eq!(
            CollectionOperation::parse(operation.as_str()),
            Some(operation)
        );
    }
    assert_eq!(CollectionOperation::parse("unknown"), None);
}

#[test]
fn units_round_trip_every_canonical_spelling() {
    assert_eq!(UnitSuffix::ALL.len(), 22);
    assert_eq!(UnitSuffix::EXECUTABLE_EXPRESSION.len(), 6);
    assert_eq!(
        UnitSuffix::ALL.map(UnitSuffix::as_str),
        [
            "s", "ms", "us", "px", "%", "deg", "fps", "hz", "khz", "db", "dbtp", "lufs", "lu", "k",
            "stops", "bps", "kbps", "mbps", "bit", "kbit", "mbit", "times",
        ]
    );
    for unit in UnitSuffix::ALL {
        assert_eq!(UnitSuffix::parse(unit.as_str()), Some(unit));
    }
    for unit in UnitSuffix::EXECUTABLE_EXPRESSION {
        assert!(unit.is_executable_expression());
    }
    assert_eq!(UnitSuffix::Milliseconds.scale(), (1, 1_000));
    assert_eq!(UnitSuffix::Kilohertz.scale(), (1_000, 1));
    assert_eq!(UnitSuffix::MegabitsPerSecond.scale(), (1_000_000, 1));
    assert_eq!(UnitSuffix::Pixels.dimension(), UnitDimension::Length);
    assert_eq!(
        UnitSuffix::DecibelsTruePeak.dimension(),
        UnitDimension::TruePeak
    );
    assert_eq!(
        UnitSuffix::Repetitions.dimension(),
        UnitDimension::Repetition
    );
    assert_eq!(UnitSuffix::parse(""), None);
    assert_eq!(UnitSuffix::parse("frames"), None);
}
