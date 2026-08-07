use std::sync::Arc;

use crate::program::expression::{
    ExactNumber as Number, ExpressionError, PrimitiveType, ResidualRuntimeValue, ResidualValue,
    ResidualizationError, Value, ValueType,
};
use crate::program::DomainType;
use veac_ir::{
    ItemId, MaterialId, SequenceId, TemporalInputSource, TemporalNodeId, TemporalParameterId,
    TemporalType, TemporalValue,
};

#[test]
fn every_closed_value_type_has_an_explicit_temporal_mapping() {
    use PrimitiveType as Primitive;
    use TemporalType as Temporal;

    for (primitive, temporal) in [
        (Primitive::Boolean, Temporal::Boolean),
        (Primitive::Integer, Temporal::Integer),
        (Primitive::Scalar, Temporal::Scalar),
        (Primitive::Percent, Temporal::Scalar),
        (Primitive::Time, Temporal::Time),
        (Primitive::Length, Temporal::Length),
        (Primitive::Angle, Temporal::Angle),
        (Primitive::Color, Temporal::Color),
        (Primitive::Text, Temporal::Text),
    ] {
        assert_eq!(
            super::super::convert::value_type(&primitive.into()),
            Some(temporal)
        );
    }
    assert_eq!(
        super::super::convert::value_type(&Primitive::Identifier.into()),
        None
    );
    for (domain, temporal) in [
        (DomainType::Vector, Temporal::Vec2),
        (DomainType::Point, Temporal::Point),
        (DomainType::Rect, Temporal::Rect),
    ] {
        assert_eq!(
            super::super::convert::value_type(&ValueType::domain(domain)),
            Some(temporal)
        );
    }
    assert_eq!(
        super::super::convert::value_type(&ValueType::domain(DomainType::Project)),
        None
    );
}

#[test]
fn every_closed_literal_converts_and_malformed_values_fail_closed() {
    let values = [
        Value::Bool(true),
        Value::Integer(7),
        Value::Scalar(Number::new(3, 2).unwrap()),
        Value::Percent(Number::integer(25)),
        Value::Time(Number::new(3, 2).unwrap()),
        Value::Length(Number::integer(8)),
        Value::Angle(Number::integer(90)),
        Value::Text(Arc::from("title")),
        Value::Color(Arc::from("#102030")),
        Value::Color(Arc::from("#10203040")),
    ];
    for value in values {
        let (value_type, converted) = super::super::convert::temporal_value(&value, 4..9).unwrap();
        assert_eq!(converted.value_type(), value_type);
    }
    for value in [
        Value::Color(Arc::from("#xyz")),
        Value::Color(Arc::from("102030")),
    ] {
        let error = super::super::convert::temporal_value(&value, 4..9).unwrap_err();
        assert_eq!(error.code(), "RESIDUAL_COLOR_LITERAL");
    }
    let oversized = Value::Time(Number::integer(i128::from(i64::MAX) + 1));
    assert_eq!(
        super::super::convert::temporal_value(&oversized, 4..9)
            .unwrap_err()
            .code(),
        "RESIDUAL_TIME_RANGE"
    );
    let unsupported = Value::Identifier(Arc::from("asset"));
    assert_eq!(
        super::super::convert::temporal_value(&unsupported, 4..9)
            .unwrap_err()
            .code(),
        "RESIDUAL_VALUE_UNSUPPORTED"
    );
}

#[test]
fn residual_models_errors_and_input_sources_expose_stable_contracts() {
    let residual = ResidualValue {
        node_id: TemporalNodeId::new(3),
        value_type: TemporalType::Angle,
    };
    assert_eq!(residual.node_id(), TemporalNodeId::new(3));
    assert_eq!(residual.value_type(), TemporalType::Angle);
    assert_eq!(
        super::super::convert::runtime_type(&ResidualRuntimeValue::Residual(residual), 1..2)
            .unwrap(),
        TemporalType::Angle
    );

    let error = ResidualizationError::expression(ExpressionError::new("E", "bad", 2..5));
    assert_eq!(error.message(), "bad");
    assert_eq!(error.span(), 2..5);
    assert_eq!(error.to_string(), "E at 2..5: bad");

    use crate::program::expression::CoreTemporalInputIdentity as Identity;
    let identities = [
        Identity::SequenceTime {
            sequence_id: SequenceId::new("seq_value_coverage").unwrap(),
        },
        Identity::ClipTime {
            item_id: ItemId::new("itm_clip_coverage").unwrap(),
        },
        Identity::SourceTime {
            source_id: MaterialId::new("med_source_coverage").unwrap(),
        },
        Identity::Frame {
            sequence_id: SequenceId::new("seq_frame_coverage").unwrap(),
        },
        Identity::Progress {
            item_id: ItemId::new("itm_progress_coverage").unwrap(),
        },
        super::support::parameter("coverage", TemporalType::Scalar),
    ];
    for identity in identities {
        let source = super::super::input::source(&identity);
        assert!(matches!(
            source,
            TemporalInputSource::Clock { .. } | TemporalInputSource::Parameter { .. }
        ));
    }
    assert!(TemporalParameterId::new("tpm_coverage").is_ok());
    assert!(matches!(
        TemporalValue::Integer { value: 1 },
        TemporalValue::Integer { .. }
    ));
}
