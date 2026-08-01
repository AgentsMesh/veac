use super::*;

#[test]
fn spring_expression_uses_the_damped_oscillator_terms() {
    let interpolation = Interpolation::Spring {
        frequency: 1.5,
        decay: 6.0,
        initial_velocity: 0.0,
    };
    let expression = easing("p", &interpolation);
    assert!(expression.contains("exp("));
    assert!(expression.contains("cos("));
    assert!(expression.contains("sin("));
    assert!(expression.contains("(p)"));
}

#[test]
fn spring_expression_preserves_tiny_restricted_parameters() {
    let interpolation = Interpolation::Spring {
        frequency: 1e-14,
        decay: 1.0,
        initial_velocity: 0.0,
    };
    let expression = easing("p", &interpolation);
    assert!(expression.contains("e-14") || expression.contains("e-13"));
    assert!(expression.contains("1.00000000000000000e0"));
}
