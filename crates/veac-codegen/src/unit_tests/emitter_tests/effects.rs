use veac_plan::canonical::*;
use veac_plan::ResolvedEffect;

use super::support::{
    assert_rgb_plane_output, bindings, emit_video_command, fixture, resolved, time,
};

mod animated;
mod plugin;

#[test]
fn emits_every_registered_video_effect_and_parameter_form() {
    let mut plan = resolved(&fixture());
    clip(&mut plan).effects = vec![
        effect(
            "fx_color",
            Effect::VideoColorAdjust {
                brightness: curve(0.2),
                contrast: constant(1.3),
                saturation: constant(1.0),
            },
        ),
        effect(
            "fx_blur",
            Effect::VideoBlur {
                radius: constant(2.0),
            },
        ),
        effect(
            "fx_sharp",
            Effect::VideoSharpen {
                amount: constant(2.0),
            },
        ),
        effect(
            "fx_vignette",
            Effect::VideoVignette {
                amount: constant(0.4),
            },
        ),
        effect(
            "fx_grain",
            Effect::VideoGrain {
                amount: constant(0.2),
            },
        ),
        effect(
            "fx_key_custom",
            Effect::VideoChromaKey {
                color: Color {
                    red: 1,
                    green: 2,
                    blue: 3,
                    alpha: 255,
                },
                similarity: constant(0.2),
                blend: constant(0.0),
            },
        ),
        effect(
            "fx_key_default",
            Effect::neutral(EffectKind::VideoChromaKey),
        ),
        effect("fx_stabilize", Effect::VideoStabilize { enabled: true }),
        effect(
            "fx_stabilize_off",
            Effect::VideoStabilize { enabled: false },
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
fn closed_effect_schema_rejects_unknown_discriminators() {
    let value = effect("fx_schema", Effect::neutral(EffectKind::VideoBlur));
    let mut json = serde_json::to_value(value).unwrap();
    json["effect"]["type"] = serde_json::json!("video_unknown");
    assert!(serde_json::from_value::<ResolvedEffect>(json).is_err());
}

#[test]
fn audio_normalize_splices_partial_ranges_and_has_a_strict_scalar_schema() {
    let mut partial = audio_plan();
    clip(&mut partial).effects = vec![ResolvedEffect {
        active_range: TimeRange::new(time(100), time(300)).unwrap(),
        ..effect(
            "fx_partial_norm",
            Effect::AudioNormalize { target_lufs: -18.0 },
        )
    }];
    let graph = graph(&partial);
    assert!(graph.contains("asplit=3"), "{graph}");
    assert!(graph.contains("atrim=start=0.166666666667:end=0.666666666667"));
    assert!(graph.contains("loudnorm=I=-18:LRA=11:TP=-1.5"));
    assert!(graph.contains("concat=n=3:v=0:a=1"));

    let value = effect(
        "fx_norm_schema",
        Effect::AudioNormalize { target_lufs: -18.0 },
    );
    let mut json = serde_json::to_value(value).unwrap();
    json["effect"]["target_lufs"] = serde_json::json!({"type": "constant", "value": -18.0});
    assert!(serde_json::from_value::<ResolvedEffect>(json).is_err());
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

fn effect(id: &str, effect: Effect) -> ResolvedEffect {
    ResolvedEffect {
        id: EffectId::new(id).unwrap(),
        active_range: TimeRange::new(time(0), time(600)).unwrap(),
        effect,
    }
}

fn constant(value: f64) -> Animatable<f64> {
    Animatable::constant(value)
}
fn curve(value: f64) -> Animatable<f64> {
    Animatable::Keyframes {
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
