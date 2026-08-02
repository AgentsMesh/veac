mod mutations;
mod owners;

use std::panic::{catch_unwind, AssertUnwindSafe};

use veac_codegen::emitter::{emit_all, CodegenErrorKind};
use veac_plan::canonical::*;
use veac_plan::{ResolvedEffect, ResolvedRenderPlan};

use self::owners::Target;
use super::support::{bindings, fixture, resolved, time};

#[test]
fn every_codegen_curve_owner_rejects_an_empty_keyframe_dto_without_panicking() {
    for target in Target::ALL {
        assert_invalid(
            owners::malformed(target),
            "PLAN_ANIMATION_INVALID",
            &format!("{target:?}"),
        );
    }
}

#[test]
fn every_codegen_curve_owner_rejects_invalid_constant_values() {
    for target in Target::ALL {
        assert_invalid(
            owners::malformed_value(target),
            "PLAN_ANIMATION_INVALID",
            &format!("{target:?} value"),
        );
    }
}

#[test]
fn malformed_keyframe_shapes_fail_closed_before_expression_generation() {
    let cases: [(&str, mutations::KeyMutation); 10] = [
        ("invalid ID", mutations::invalid_id),
        ("zero timebase", mutations::zero_timebase),
        ("negative time", mutations::negative_time),
        ("foreign timebase", mutations::foreign_timebase),
        ("equal time", mutations::equal_time),
        ("descending time", mutations::descending_time),
        ("outside clip", mutations::outside_clip),
        ("unsafe time", mutations::unsafe_time),
        ("invalid easing", mutations::invalid_easing),
        ("invalid value", mutations::invalid_value),
    ];
    for (name, mutate) in cases {
        let mut plan = curve_plan();
        let duration = clip(&mut plan).record_range.duration;
        let frames = frames(&mut plan);
        mutate(frames, duration);
        assert_invalid(plan, "PLAN_ANIMATION_INVALID", name);
    }
}

#[test]
fn duplicate_ids_fail_closed_and_a_single_endpoint_keyframe_remains_valid() {
    let mut duplicate = curve_plan();
    let duplicate_frames = frames(&mut duplicate);
    duplicate_frames[1].id = duplicate_frames[0].id.clone();
    assert_invalid(duplicate, "PLAN_KEYFRAME_DUPLICATE", "duplicate ID");

    let mut endpoint = curve_plan();
    let duration = clip(&mut endpoint).record_range.duration;
    *frames(&mut endpoint) = [key("kf_endpoint", duration.value, 1.0)].to_vec();
    emit_all(&endpoint, &bindings(&endpoint)).expect("one endpoint keyframe is meaningful");
}

#[test]
fn malformed_effect_domains_and_text_stagger_fail_closed() {
    let duration = curve_plan().sequences[0].tracks[0].clips[0]
        .record_range
        .duration;
    for (name, range) in [
        ("negative start", range(-1, 1, 600)),
        ("zero duration", range(0, 0, 600)),
        ("foreign timebase", range(0, 1, 30)),
        ("outside clip", range(0, duration.value + 1, 600)),
        ("zero timebase", range(0, 1, 0)),
    ] {
        assert_invalid(effect_plan(range), "PLAN_EFFECT_RANGE_INVALID", name);
    }
    assert_invalid(
        owners::invalid_stagger(),
        "PLAN_ANIMATION_INVALID",
        "text stagger",
    );
}

#[test]
fn malformed_crop_viewports_fail_closed_as_untrusted_plan_data() {
    let mut invalid_value = curve_plan();
    clip(&mut invalid_value)
        .visual
        .as_mut()
        .unwrap()
        .transform
        .crop = Some(Animatable::constant(Rect {
        x: 0.8,
        y: 0.0,
        width: 0.4,
        height: 1.0,
    }));
    assert_invalid(invalid_value, "PLAN_ANIMATION_INVALID", "crop bounds");

    let mut empty = curve_plan();
    clip(&mut empty).visual.as_mut().unwrap().transform.crop =
        Some(Animatable::Keyframes { keyframes: vec![] });
    assert_invalid(empty, "PLAN_ANIMATION_INVALID", "empty crop curve");

    let mut duplicate = curve_plan();
    clip(&mut duplicate).visual.as_mut().unwrap().transform.crop = Some(Animatable::Keyframes {
        keyframes: vec![Keyframe {
            id: KeyframeId::new("kf_preflight_a").unwrap(),
            time: time(0),
            value: Rect {
                x: 0.0,
                y: 0.0,
                width: 0.5,
                height: 1.0,
            },
            interpolation: Interpolation::Linear,
        }],
    });
    assert_invalid(duplicate, "PLAN_KEYFRAME_DUPLICATE", "crop key ID");
}

fn assert_invalid(plan: ResolvedRenderPlan, code: &'static str, name: &str) {
    let result = catch_unwind(AssertUnwindSafe(|| emit_all(&plan, &bindings(&plan))))
        .unwrap_or_else(|_| panic!("{name} panicked"));
    let error = result.expect_err(name);
    let diagnostic = error
        .diagnostics()
        .iter()
        .find(|value| value.code == code)
        .unwrap_or_else(|| panic!("{name} missing {code}: {error}"));
    assert_eq!(diagnostic.kind, CodegenErrorKind::InvalidPlan, "{name}");
}

fn curve_plan() -> ResolvedRenderPlan {
    let mut plan = resolved(&fixture());
    clip(&mut plan).visual.as_mut().unwrap().opacity = Animatable::Keyframes {
        keyframes: vec![
            key("kf_preflight_a", 0, 0.0),
            key("kf_preflight_b", 600, 1.0),
        ],
    };
    plan
}

fn effect_plan(active_range: TimeRange) -> ResolvedRenderPlan {
    let mut plan = resolved(&fixture());
    clip(&mut plan).effects.push(ResolvedEffect {
        id: EffectId::new("fx_preflight").unwrap(),
        effect_type: "video.color_adjust".to_owned(),
        active_range,
        parameters: Default::default(),
    });
    plan
}

fn clip(plan: &mut ResolvedRenderPlan) -> &mut veac_plan::ResolvedClip {
    &mut plan.sequences[0].tracks[0].clips[0]
}

fn frames(plan: &mut ResolvedRenderPlan) -> &mut Vec<Keyframe<f64>> {
    match &mut clip(plan).visual.as_mut().unwrap().opacity {
        Animatable::Keyframes { keyframes } => keyframes,
        _ => unreachable!(),
    }
}

fn key(id: &str, at: i64, value: f64) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value,
        interpolation: Interpolation::Linear,
    }
}

fn range(start: i64, duration: i64, timescale: u32) -> TimeRange {
    TimeRange {
        start: RationalTime {
            value: start,
            timescale,
        },
        duration: RationalTime {
            value: duration,
            timescale,
        },
    }
}
