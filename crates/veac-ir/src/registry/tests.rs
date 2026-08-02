use super::*;

#[test]
fn parameter_spec_builders_preserve_runtime_schema_contracts() {
    assert_eq!(
        number("amount", -1.0, 2.0),
        ParameterSpec {
            name: "amount",
            value_type: ParameterType::Number,
            minimum: Some(-1.0),
            maximum: Some(2.0),
            supports_curve: true,
        }
    );
    assert_eq!(
        boolean("enabled"),
        ParameterSpec {
            name: "enabled",
            value_type: ParameterType::Boolean,
            minimum: None,
            maximum: None,
            supports_curve: false,
        }
    );
}
