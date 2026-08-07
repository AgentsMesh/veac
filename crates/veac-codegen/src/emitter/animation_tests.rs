use super::*;
use veac_plan::canonical::*;

use super::temporal::tests::support::{node, plan, program};

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

#[test]
fn temporal_composite_bindings_reach_every_animation_sink_accessor() {
    let (plan, binding) = literal_plan(TemporalValue::Vec2 {
        value: Vec2 { x: 0.25, y: 0.75 },
    });
    let owner = clip_owner(&plan);
    let value = Animatable::Binding {
        binding_id: binding,
    };
    assert_eq!(vec_x(&plan, owner, &value, "t"), "0.25");
    assert_eq!(vec_y(&plan, owner, &value, "t"), "0.75");

    let (plan, binding) = literal_plan(TemporalValue::Point {
        value: Point {
            x: Length {
                value: 0.25,
                unit: LengthUnit::Normalized,
            },
            y: Length {
                value: 50.0,
                unit: LengthUnit::Percent,
            },
        },
    });
    let owner = clip_owner(&plan);
    let value = Animatable::Binding {
        binding_id: binding,
    };
    assert_eq!(point_x(&plan, owner, &value, "t", "W"), "(W)*(0.25)");
    assert_eq!(point_y(&plan, owner, &value, "t", "H"), "(H)*(0.5)");

    let (plan, binding) = literal_plan(TemporalValue::Rect {
        value: Rect {
            x: 0.1,
            y: 0.2,
            width: 0.3,
            height: 0.4,
        },
    });
    let owner = clip_owner(&plan);
    let value = Animatable::Binding {
        binding_id: binding,
    };
    assert_eq!(rect_x(&plan, owner, &value, "t"), "0.1");
    assert_eq!(rect_y(&plan, owner, &value, "t"), "0.2");
    assert_eq!(rect_width(&plan, owner, &value, "t"), "0.3");
    assert_eq!(rect_height(&plan, owner, &value, "t"), "0.4");
}

fn literal_plan(value: TemporalValue) -> (veac_plan::ResolvedRenderPlan, TemporalBindingId) {
    let value_type = value.value_type();
    plan(
        program(
            Vec::new(),
            vec![node(0, value_type, TemporalNodeKind::Literal { value })],
            0,
            value_type,
        ),
        Vec::new(),
        Vec::new(),
    )
}

fn clip_owner(plan: &veac_plan::ResolvedRenderPlan) -> ProcessOwner<'_> {
    ProcessOwner::clip(&plan.sequences[0].tracks[0].clips[0])
}
