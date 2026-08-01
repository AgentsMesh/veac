use veac_codegen::emitter::CodegenErrorKind;
use veac_plan::canonical::*;
use veac_plan::ResolvedEffect;

use super::support::{
    assert_rgb_plane_output, bindings, emit_video_command, fixture, resolved, time,
};

mod animated;

#[test]
fn emits_every_registered_video_effect_and_parameter_form() {
    let mut plan = resolved(&fixture());
    clip(&mut plan).effects = vec![
        effect(
            "fx_color",
            "video.color_adjust",
            [("brightness", curve(0.2)), ("contrast", number(1.3))],
        ),
        effect("fx_blur", "video.blur", [("radius", number(2.0))]),
        effect("fx_sharp", "video.sharpen", [("amount", number(2.0))]),
        effect("fx_vignette", "video.vignette", [("amount", number(0.4))]),
        effect("fx_grain", "video.grain", [("amount", number(0.2))]),
        effect(
            "fx_key_custom",
            "video.chroma_key",
            [
                (
                    "color",
                    ParameterValue::Color {
                        value: Color {
                            red: 1,
                            green: 2,
                            blue: 3,
                            alpha: 255,
                        },
                    },
                ),
                ("similarity", number(0.2)),
            ],
        ),
        effect("fx_key_default", "video.chroma_key", []),
        effect(
            "fx_stabilize",
            "video.stabilize",
            [("enabled", ParameterValue::Boolean { value: true })],
        ),
        effect(
            "fx_stabilize_off",
            "video.stabilize",
            [("enabled", ParameterValue::Boolean { value: false })],
        ),
    ];
    let graph = graph(&plan);
    for marker in [
        "eq=brightness=",
        "gblur@",
        "sigma=2",
        "cas@",
        "strength=0.2",
        "vignette=angle='PI/(5-4*(0.4))':eval=frame",
        "noise=alls='0.2*100'",
        "color=0x010203:similarity=0.2:blend=0",
        "color=0x00FF00:similarity=0.1:blend=0",
        "vidstabtransform=input=",
    ] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
    assert_rgb_plane_output(&graph, "effectcolorv");
    assert_rgb_plane_output(&graph, "effectoutputcolorv");
    assert!(graph.contains("clip((t-0)/1"));
    assert!(graph.contains("enable='gte(t,0)*lt(t,1)'"));
    let command = emit_video_command(&plan, &bindings(&plan)).unwrap();
    assert_eq!(command.preparations.len(), 1);
    let analysis = command.preparations[0]
        .command
        .filter_graph
        .as_deref()
        .unwrap();
    assert!(analysis.contains("vidstabdetect=result="), "{analysis}");
    assert_eq!(graph.matches("vidstabtransform").count(), 1);
}

#[test]
fn unknown_effects_fail_closed() {
    let mut plan = resolved(&fixture());
    clip(&mut plan).effects = vec![effect("fx_unknown", "video.unknown", [])];
    assert_effect_error(&plan, "PLAN_EFFECT_UNKNOWN");
}

#[test]
fn audio_normalize_splices_partial_ranges_and_rejects_curved_target() {
    let mut partial = audio_plan();
    clip(&mut partial).effects = vec![ResolvedEffect {
        active_range: TimeRange::new(time(100), time(300)).unwrap(),
        ..effect(
            "fx_partial_norm",
            "audio.normalize",
            [("target_lufs", number(-18.0))],
        )
    }];
    let graph = graph(&partial);
    assert!(graph.contains("asplit=3"), "{graph}");
    assert!(graph.contains("atrim=start=0.166666666667:end=0.666666666667"));
    assert!(graph.contains("loudnorm=I=-18:LRA=11:TP=-1.5"));
    assert!(graph.contains("concat=n=3:v=0:a=1"));

    let mut curved = audio_plan();
    clip(&mut curved).effects = vec![effect(
        "fx_curved_norm",
        "audio.normalize",
        [("target_lufs", curve(-18.0))],
    )];
    assert_effect_error(&curved, "PLAN_EFFECT_PARAMETER_INVALID");
}

fn audio_plan() -> veac_plan::ResolvedRenderPlan {
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

fn effect<const N: usize>(
    id: &str,
    kind: &str,
    values: [(&str, ParameterValue); N],
) -> ResolvedEffect {
    ResolvedEffect {
        id: EffectId::new(id).unwrap(),
        effect_type: kind.to_owned(),
        active_range: TimeRange::new(time(0), time(600)).unwrap(),
        parameters: values
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    }
}

fn number(value: f64) -> ParameterValue {
    ParameterValue::Number { value }
}
fn curve(value: f64) -> ParameterValue {
    ParameterValue::NumberCurve {
        value: Animatable::Keyframes {
            keyframes: vec![
                Keyframe {
                    id: KeyframeId::new("kf_curve_a").unwrap(),
                    time: time(0),
                    value: 0.0,
                    interpolation: Interpolation::Linear,
                },
                Keyframe {
                    id: KeyframeId::new("kf_curve_b").unwrap(),
                    time: time(600),
                    value,
                    interpolation: Interpolation::Linear,
                },
            ],
        },
    }
}
fn clip(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut veac_plan::ResolvedClip {
    &mut plan.sequences[0].tracks[0].clips[0]
}
fn graph(plan: &veac_plan::ResolvedRenderPlan) -> String {
    emit_video_command(plan, &bindings(plan))
        .unwrap()
        .filter_graph
        .unwrap()
}
fn assert_effect_error(plan: &veac_plan::ResolvedRenderPlan, code: &str) {
    let error = emit_video_command(plan, &bindings(plan)).unwrap_err();
    assert_eq!(error.diagnostics()[0].kind, CodegenErrorKind::InvalidPlan);
    assert_eq!(error.diagnostics()[0].code, code);
}
