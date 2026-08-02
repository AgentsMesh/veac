use veac_codegen::emitter::{emit_all, CodegenErrorKind};
use veac_plan::canonical::*;
use veac_plan::ResolvedEffect;

use super::support::{bindings, fixture, resolved, time};

#[test]
fn resolved_effect_rejects_spring_extrema_outside_parameter_bounds() {
    let mut plan = resolved(&fixture());
    let duration = plan.sequences[0].tracks[0].clips[0].record_range.duration;
    plan.sequences[0].tracks[0].clips[0]
        .effects
        .push(ResolvedEffect {
            id: EffectId::new("fx_threshold_spring").unwrap(),
            effect_type: "video.luma_key".to_owned(),
            active_range: TimeRange::new(time(0), duration).unwrap(),
            parameters: [("threshold".to_owned(), spring_curve())].into(),
        });
    let error = emit_all(&plan, &bindings(&plan)).expect_err("spring overshoot must fail");
    assert!(error.diagnostics().iter().any(|diagnostic| {
        diagnostic.kind == CodegenErrorKind::InvalidPlan
            && diagnostic.code == "PLAN_EFFECT_PARAMETER_INVALID"
    }));
}

fn spring_curve() -> ParameterValue {
    ParameterValue::NumberCurve {
        value: Animatable::Keyframes {
            keyframes: vec![
                key("kf_threshold_start", 0, 0.1),
                key("kf_threshold_end", 600, 0.9),
            ],
        },
    }
}

fn key(id: &str, at: i64, value: f64) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value,
        interpolation: Interpolation::Spring {
            frequency: 1.5,
            decay: 6.0,
            initial_velocity: 0.0,
        },
    }
}
