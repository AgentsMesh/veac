use crate::PrimitiveType;

#[test]
fn primitive_tokens_round_trip_and_numeric_capability_is_exact() {
    let numeric = [
        PrimitiveType::Integer,
        PrimitiveType::Scalar,
        PrimitiveType::Time,
        PrimitiveType::Length,
        PrimitiveType::Percent,
        PrimitiveType::Angle,
    ];
    for (value, token) in PrimitiveType::ALL.into_iter().zip(PrimitiveType::TOKENS) {
        assert_eq!(value.as_str(), *token);
        assert_eq!(PrimitiveType::parse(token), Some(value));
        assert_eq!(value.to_string(), *token);
        assert_eq!(value.is_numeric(), numeric.contains(&value));
    }
    for token in ["", "INT", "number", "time "] {
        assert_eq!(PrimitiveType::parse(token), None);
    }
}
