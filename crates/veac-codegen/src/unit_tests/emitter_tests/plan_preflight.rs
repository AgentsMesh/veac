use std::panic::{catch_unwind, AssertUnwindSafe};

use veac_codegen::emitter::{emit_all, CodegenErrorKind};
use veac_plan::canonical::*;
use veac_plan::{ResolvedEffect, ResolvedRenderPlan, ResolvedSidechain};

use super::support::{add_transition, bindings, fixture, resolved, time, transition_visual};

mod audio;
mod temporal;

type EffectMutation = fn(&mut ResolvedEffect);
type EffectCase = (&'static str, EffectMutation);

#[test]
fn untrusted_effect_contracts_fail_in_preflight() {
    let cases: [EffectCase; 5] = [
        ("PLAN_EFFECT_PARAMETER_INVALID", |value| {
            value.effect = Effect::VideoBlur {
                radius: Animatable::constant(101.0),
            };
        }),
        ("PLAN_EFFECT_PARAMETER_INVALID", |value| {
            value.effect = Effect::VideoBlur {
                radius: Animatable::constant(f64::NAN),
            };
        }),
        ("PLAN_EFFECT_PARAMETER_INVALID", |value| {
            value.effect = Effect::AudioNormalize { target_lufs: -4.0 };
        }),
        ("PLAN_PLUGIN_EFFECT_DESCRIPTOR_INVALID", |value| {
            value.effect = Effect::VideoPluginReferenceMonochromeV1 {
                descriptor_digest: PluginEffectDigest::new("0".repeat(64)).unwrap(),
                amount: Animatable::constant(0.5),
            };
        }),
        ("PLAN_EFFECT_ID_INVALID", |value| {
            value.id = serde_json::from_str("\"invalid\"").unwrap();
        }),
    ];
    for (code, mutate) in cases {
        let mut plan = resolved(&fixture());
        let mut value = effect("fx_preflight_contract", EffectKind::VideoBlur);
        mutate(&mut value);
        plan.sequences[0].tracks[0].clips[0].effects.push(value);
        assert_code(&plan, code);
    }

    let mut duplicate = resolved(&fixture());
    let value = effect("fx_preflight_duplicate", EffectKind::VideoBlur);
    duplicate.sequences[0].tracks[0].clips[0].effects = vec![value.clone(), value];
    assert_code(&duplicate, "PLAN_EFFECT_DUPLICATE");
}

#[test]
fn resolved_effect_schema_rejects_unknown_mismatched_and_extra_fields() {
    let value = effect("fx_schema_contract", EffectKind::VideoBlur);
    let base = serde_json::to_value(value).unwrap();
    for mutate in [
        unknown_effect as fn(&mut serde_json::Value),
        mismatched_parameter,
        extra_parameter,
        missing_parameter,
    ] {
        let mut candidate = base.clone();
        mutate(&mut candidate);
        assert!(serde_json::from_value::<ResolvedEffect>(candidate).is_err());
    }
}

#[test]
fn untrusted_structure_header_and_input_facts_fail_closed() {
    let mut header = resolved(&fixture());
    header.header.source.timebase = 0;
    assert_code(&header, "PLAN_HEADER_INVALID");

    let mut settings = resolved(&fixture());
    settings.sequences[0].settings.width = 0;
    assert_code(&settings, "PLAN_STRUCTURE_INVALID");

    let mut duplicate = resolved(&fixture());
    let mut track = duplicate.sequences[0].tracks[0].clone();
    track.order += 1;
    track.source_order += 1;
    duplicate.sequences[0].tracks.push(track);
    assert_code(&duplicate, "PLAN_STRUCTURE_INVALID");

    let mut input = resolved(&fixture());
    input.inputs[0].video.as_mut().unwrap().info.width = 0;
    assert_code(&input, "PLAN_INPUT_FACTS_INVALID");
}

#[test]
fn untrusted_transition_handles_fail_closed() {
    let mut project = fixture();
    let track = &mut project.project.sequences[0].tracks[0];
    track.clips[0].visual = Some(transition_visual());
    let transition = Transition {
        kind: TransitionKind::Dissolve,
        duration: time(120),
        alignment: TransitionAlignment::Centered,
    };
    let mut incoming = track.clips[0].clone();
    incoming.id = ItemId::new("itm_preflight_incoming").unwrap();
    incoming.record_range.start = time(480);
    track.clips.push(incoming);
    add_transition(
        &mut project,
        "seq_main",
        "itm_video",
        "itm_preflight_incoming",
        transition,
    );
    let mut plan = resolved(&project);
    plan.sequences[0].tracks[0].transitions[0]
        .incoming_range
        .duration
        .value += 1;
    assert_code(&plan, "PLAN_TRANSITION_INVALID");
}

fn effect(id: &str, kind: EffectKind) -> ResolvedEffect {
    ResolvedEffect {
        id: EffectId::new(id).unwrap(),
        active_range: TimeRange::new(time(0), time(600)).unwrap(),
        effect: Effect::neutral(kind),
    }
}

fn unknown_effect(value: &mut serde_json::Value) {
    value["effect"]["type"] = serde_json::json!("video_missing");
}

fn mismatched_parameter(value: &mut serde_json::Value) {
    value["effect"]["radius"] = serde_json::json!(true);
}

fn extra_parameter(value: &mut serde_json::Value) {
    value["effect"]["missing"] = serde_json::json!(1.0);
}

fn missing_parameter(value: &mut serde_json::Value) {
    value["effect"].as_object_mut().unwrap().remove("radius");
}

fn audio_plan() -> ResolvedRenderPlan {
    let mut project = fixture();
    project.project.render_configs[0]
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    project.project.sequences[0].tracks[0].clips[0].audio = Some(AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
    });
    resolved(&project)
}

fn audio(plan: &mut ResolvedRenderPlan) -> &mut veac_plan::EffectiveAudioProperties {
    plan.sequences[0].tracks[0].clips[0].audio.as_mut().unwrap()
}

fn assert_code(plan: &ResolvedRenderPlan, code: &'static str) {
    let result = catch_unwind(AssertUnwindSafe(|| emit_all(plan, &bindings(plan))));
    let error = result.expect("preflight must not panic").expect_err(code);
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|value| { value.kind == CodegenErrorKind::InvalidPlan && value.code == code }),
        "missing {code}: {error}"
    );
}
