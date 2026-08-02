use std::panic::{catch_unwind, AssertUnwindSafe};

use veac_codegen::emitter::{emit_all, CodegenErrorKind};
use veac_plan::canonical::*;
use veac_plan::{ResolvedEffect, ResolvedRenderPlan, ResolvedSidechain};

use super::support::{add_transition, bindings, fixture, resolved, time};

mod audio;

type EffectMutation = fn(&mut ResolvedEffect);
type EffectCase = (&'static str, EffectMutation);

#[test]
fn untrusted_effect_contracts_fail_in_preflight() {
    let cases: [EffectCase; 6] = [
        ("PLAN_EFFECT_UNKNOWN", |value| {
            value.effect_type = "video.missing".into()
        }),
        ("PLAN_EFFECT_PARAMETER_INVALID", |value| {
            value
                .parameters
                .insert("missing".into(), ParameterValue::Number { value: 1.0 });
        }),
        ("PLAN_EFFECT_PARAMETER_INVALID", |value| {
            value
                .parameters
                .insert("radius".into(), ParameterValue::Boolean { value: true });
        }),
        ("PLAN_EFFECT_PARAMETER_INVALID", |value| {
            value
                .parameters
                .insert("radius".into(), ParameterValue::Number { value: 101.0 });
        }),
        ("PLAN_EFFECT_PARAMETER_INVALID", |value| {
            value.effect_type = "audio.normalize".into();
            value.parameters.insert(
                "target_lufs".into(),
                ParameterValue::NumberCurve {
                    value: Animatable::constant(-18.0),
                },
            );
        }),
        ("PLAN_EFFECT_ID_INVALID", |value| {
            value.id = serde_json::from_str("\"invalid\"").unwrap();
        }),
    ];
    for (code, mutate) in cases {
        let mut plan = resolved(&fixture());
        let mut value = effect("fx_preflight_contract", "video.blur");
        mutate(&mut value);
        plan.sequences[0].tracks[0].clips[0].effects.push(value);
        assert_code(&plan, code);
    }

    let mut duplicate = resolved(&fixture());
    let value = effect("fx_preflight_duplicate", "video.blur");
    duplicate.sequences[0].tracks[0].clips[0].effects = vec![value.clone(), value];
    assert_code(&duplicate, "PLAN_EFFECT_DUPLICATE");
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
    let transition = Transition {
        kind: TransitionKind::Dissolve,
        duration: time(120),
        alignment: TransitionAlignment::Centered,
    };
    let mut incoming = track.clips[0].clone();
    incoming.id = ItemId::new("itm_preflight_incoming").unwrap();
    incoming.record_range.start = time(600);
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
        .incoming_handle
        .duration
        .value += 1;
    assert_code(&plan, "PLAN_TRANSITION_INVALID");
}

fn effect(id: &str, kind: &str) -> ResolvedEffect {
    ResolvedEffect {
        id: EffectId::new(id).unwrap(),
        effect_type: kind.into(),
        active_range: TimeRange::new(time(0), time(600)).unwrap(),
        parameters: Default::default(),
    }
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
