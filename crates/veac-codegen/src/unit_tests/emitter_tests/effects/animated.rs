use std::collections::BTreeSet;

use veac_plan::canonical::*;

use super::{clip, effect, graph};
use crate::unit_tests::emitter_tests::support::{fixture, resolved, time};

mod sink_bounds;

#[test]
fn every_curve_capable_video_effect_has_a_frame_evaluated_backend_path() {
    for (kind, value, marker) in [
        (EffectKind::VideoBlur, 4.0, "gblur@"),
        (EffectKind::VideoSharpen, 3.0, "cas@"),
        (EffectKind::VideoVignette, 0.8, "eval=frame"),
        (EffectKind::VideoGrain, 0.7, "blend=all_expr"),
        (EffectKind::VideoChromaKey, 0.5, "colorkey@"),
        (EffectKind::VideoLumaKey, 0.5, "lumakey@"),
        (EffectKind::VideoChromaSpill, 0.8, "despill@"),
    ] {
        let mut plan = resolved(&fixture());
        clip(&mut plan).effects = vec![effect("fx_animated", with_curve(kind, animated(value)))];
        let rendered = graph(&plan);
        assert!(rendered.contains(marker), "missing {marker}: {rendered}");
        if !matches!(kind, EffectKind::VideoVignette | EffectKind::VideoGrain) {
            assert!(rendered.contains("sendcmd=c='0-1 [expr]"), "{rendered}");
        }
    }
}

#[test]
fn cubic_runtime_commands_escape_nested_command_delimiters() {
    let mut plan = resolved(&fixture());
    let mut value = animated(0.6);
    let Animatable::Keyframes { keyframes } = &mut value else {
        unreachable!()
    };
    keyframes[0].interpolation = Interpolation::CubicBezier {
        x1: 0.2,
        y1: 0.1,
        x2: 0.8,
        y2: 0.9,
    };
    clip(&mut plan).effects = vec![effect(
        "fx_cubic_key",
        with_curve(EffectKind::VideoChromaKey, value),
    )];
    let rendered = graph(&plan);
    assert!(rendered.contains("st(1\\\\,"), "{rendered}");
    assert!(rendered.contains("\\\\;"), "{rendered}");
}

#[test]
fn runtime_filter_instances_are_bounded_deterministic_and_distinct() {
    const FIRST: &str = "fx_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const SECOND: &str = "fx_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab";
    let mut plan = resolved(&fixture());
    let mut second_curve = animated(8.0);
    let Animatable::Keyframes { keyframes } = &mut second_curve else {
        unreachable!()
    };
    keyframes[0].id = KeyframeId::new("kf_second_start").unwrap();
    keyframes[1].id = KeyframeId::new("kf_second_end").unwrap();
    clip(&mut plan).effects = vec![
        effect(FIRST, with_curve(EffectKind::VideoBlur, animated(4.0))),
        effect(SECOND, with_curve(EffectKind::VideoBlur, second_curve)),
    ];
    let first = instances(&graph(&plan));
    let second = instances(&graph(&plan));
    assert_eq!(first, second, "instance names must be deterministic");
    assert_eq!(first.len(), 2, "different effect IDs must not alias");
    for value in first {
        assert_eq!(value.len(), 37, "FFmpeg target must remain bounded");
        assert!(value.starts_with("veac_"));
        assert!(value[5..].bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
}

fn instances(graph: &str) -> BTreeSet<String> {
    graph
        .split("gblur@")
        .skip(1)
        .map(|suffix| {
            suffix
                .chars()
                .take_while(|value| value.is_ascii_alphanumeric() || *value == '_')
                .collect()
        })
        .collect()
}

fn animated(end: f64) -> Animatable<f64> {
    Animatable::Keyframes {
        keyframes: vec![
            Keyframe {
                id: KeyframeId::new("kf_effect_start").unwrap(),
                time: time(0),
                value: if end > 0.00001 { 0.00001 } else { 0.0 },
                interpolation: Interpolation::Linear,
            },
            Keyframe {
                id: KeyframeId::new("kf_effect_end").unwrap(),
                time: time(600),
                value: end,
                interpolation: Interpolation::Linear,
            },
        ],
    }
}

fn with_curve(kind: EffectKind, value: Animatable<f64>) -> Effect {
    let parameter = match kind {
        EffectKind::VideoBlur => EffectParameter::Radius,
        EffectKind::VideoChromaKey => EffectParameter::Similarity,
        EffectKind::VideoLumaKey => EffectParameter::Threshold,
        EffectKind::VideoChromaSpill
        | EffectKind::VideoSharpen
        | EffectKind::VideoVignette
        | EffectKind::VideoGrain => EffectParameter::Amount,
        _ => unreachable!("fixture only covers curve-capable video effects"),
    };
    with_parameter(kind, parameter, value)
}

fn with_parameter(kind: EffectKind, parameter: EffectParameter, value: Animatable<f64>) -> Effect {
    let mut effect = Effect::neutral(kind);
    assert_eq!(
        effect.set_parameter(parameter, EffectParameterValue::Curve(value)),
        Some(true)
    );
    effect
}
