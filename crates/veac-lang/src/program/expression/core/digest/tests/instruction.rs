use sha2::{Digest, Sha256};

use super::super::{instruction, value};
use crate::program::expression::{
    ArithmeticOperator, BuiltinFunction, CollectionOperation, ComparisonOperator, CoreCallTarget,
    CoreInstructionKind as Kind, CoreTemporalComposeOperation as Compose,
    CoreTemporalProjectOperation as Project, CoreUnaryOperator, EqualityOperator, FunctionId,
    Value, ValueId,
};
use crate::program::{FieldIndex, TypeId, VariantIndex};

fn encoded(kind: &Kind) -> [u8; 32] {
    let mut digest = Sha256::new();
    instruction::encode(&mut digest, kind);
    digest.finalize().into()
}

#[test]
fn instruction_digest_covers_every_closed_instruction_and_operator() {
    let id = ValueId::new(1);
    let ids = vec![ValueId::new(1), ValueId::new(2)];
    let mut kinds = vec![
        Kind::Literal(Value::Integer(7)),
        Kind::Input(crate::program::expression::InputId::new(2)),
        Kind::Parameter(3),
        Kind::Capture(4),
        Kind::LocalInit {
            slot: crate::program::expression::LocalSlotId::new(5),
            value: id,
        },
        Kind::LocalSet {
            slot: crate::program::expression::LocalSlotId::new(5),
            value: id,
        },
        Kind::LocalGet {
            slot: crate::program::expression::LocalSlotId::new(5),
        },
        Kind::Closure {
            definition: crate::program::expression::ClosureDefinitionId::new(5),
            captures: ids.clone(),
        },
        Kind::Invoke {
            callee: id,
            arguments: ids.clone(),
        },
        Kind::Call {
            target: CoreCallTarget::Builtin(BuiltinFunction::Min),
            arguments: ids.clone(),
        },
        Kind::Call {
            target: CoreCallTarget::User(FunctionId::from_bytes([9; 32])),
            arguments: ids.clone(),
        },
        Kind::List {
            elements: ids.clone(),
        },
        Kind::Tuple {
            elements: ids.clone(),
        },
        Kind::Range {
            start: id,
            end: ValueId::new(2),
            step: None,
        },
        Kind::Range {
            start: id,
            end: ValueId::new(2),
            step: Some(ValueId::new(3)),
        },
        Kind::MapBegin { entries: 2 },
        Kind::MapKey {
            builder: id,
            key: ValueId::new(2),
            ordinal: 3,
        },
        Kind::MapValue {
            pending: id,
            value: ValueId::new(2),
        },
        Kind::MapFinish { builder: id },
        Kind::StructConstruct {
            type_id: TypeId::from_bytes([1; 32]),
            fields: ids.clone(),
        },
        Kind::StructProject {
            structure: id,
            field: FieldIndex::new(2),
        },
        Kind::EnumConstruct {
            type_id: TypeId::from_bytes([2; 32]),
            variant: VariantIndex::new(3),
            fields: ids.clone(),
        },
        Kind::DomainConstruct {
            opcode: 0x1234,
            operands: ids.clone(),
        },
        Kind::GraphEmit {
            opcode: 0x2345,
            operands: ids.clone(),
        },
    ];
    for operator in [
        CoreUnaryOperator::Positive,
        CoreUnaryOperator::Negative,
        CoreUnaryOperator::Not,
    ] {
        kinds.push(Kind::Unary {
            operator,
            operand: id,
        });
    }
    for operator in [
        ArithmeticOperator::Add,
        ArithmeticOperator::Subtract,
        ArithmeticOperator::Multiply,
        ArithmeticOperator::Divide,
    ] {
        kinds.push(Kind::Arithmetic {
            operator,
            left: id,
            right: ValueId::new(2),
        });
    }
    for operator in [
        ComparisonOperator::Less,
        ComparisonOperator::LessEqual,
        ComparisonOperator::Greater,
        ComparisonOperator::GreaterEqual,
    ] {
        kinds.push(Kind::Compare {
            operator,
            left: id,
            right: ValueId::new(2),
        });
    }
    for operator in [EqualityOperator::Equal, EqualityOperator::NotEqual] {
        kinds.push(Kind::Equal {
            operator,
            left: id,
            right: ValueId::new(2),
        });
    }
    for (operation, initial) in [
        (CollectionOperation::Map, None),
        (CollectionOperation::Filter, None),
        (CollectionOperation::Fold, Some(ValueId::new(3))),
    ] {
        kinds.push(Kind::Collection {
            operation,
            iterable: id,
            initial,
            callable: ValueId::new(2),
        });
    }
    for operation in [Compose::Vec2, Compose::Point, Compose::Rect, Compose::Color] {
        kinds.push(Kind::TemporalCompose {
            operation,
            operands: ids.clone(),
        });
    }
    for operation in [
        Project::Vec2X,
        Project::Vec2Y,
        Project::PointX,
        Project::PointY,
        Project::RectX,
        Project::RectY,
        Project::RectWidth,
        Project::RectHeight,
        Project::ColorRed,
        Project::ColorGreen,
        Project::ColorBlue,
        Project::ColorAlpha,
    ] {
        kinds.push(Kind::TemporalProject {
            operation,
            value: id,
        });
    }
    let outputs = kinds
        .iter()
        .map(encoded)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(outputs.len(), kinds.len());
    let mut probe = Sha256::new();
    value::encode(&mut probe, &Value::Bool(true));
}
