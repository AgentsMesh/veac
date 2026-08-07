use veac_plan::canonical::*;
use veac_plan::ResolvedEffect;

use super::support::{bindings, emit_video_command, fixture, resolved, time};

#[test]
fn luma_key_inversion_and_blue_spill_suppression_emit_typed_filters() {
    let mut plan = resolved(&fixture());
    clip(&mut plan).effects = vec![
        effect(
            "fx_luma",
            Effect::VideoLumaKey {
                threshold: Animatable::constant(0.2),
                tolerance: Animatable::constant(0.1),
                softness: Animatable::constant(0.05),
                invert: true,
            },
        ),
        effect(
            "fx_spill",
            Effect::VideoChromaSpill {
                color: Color {
                    red: 0,
                    green: 10,
                    blue: 255,
                    alpha: 255,
                },
                amount: Animatable::constant(0.7),
                range: Animatable::constant(0.3),
            },
        ),
    ];
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    assert!(graph.contains("lumakey@"));
    assert!(graph.contains("threshold=0.2:tolerance=0.1:softness=0.05"));
    assert!(graph.contains("a='65535-alpha(X\\,Y)'"));
    assert!(graph.contains("despill@"));
    assert!(graph.contains("type=blue:mix=0.7:expand=0.3"));
}

#[test]
fn animated_luma_scalar_uses_runtime_filter_commands() {
    let mut plan = resolved(&fixture());
    clip(&mut plan).effects = vec![ResolvedEffect {
        id: EffectId::new("fx_luma_curve").unwrap(),
        active_range: TimeRange::new(time(0), time(600)).unwrap(),
        effect: Effect::VideoLumaKey {
            threshold: Animatable::Keyframes {
                keyframes: vec![
                    Keyframe {
                        id: KeyframeId::new("kf_luma_start").unwrap(),
                        time: time(0),
                        value: 0.2,
                        interpolation: Interpolation::Linear,
                    },
                    Keyframe {
                        id: KeyframeId::new("kf_luma_end").unwrap(),
                        time: time(600),
                        value: 0.4,
                        interpolation: Interpolation::Linear,
                    },
                ],
            },
            tolerance: Animatable::constant(0.01),
            softness: Animatable::constant(0.0),
            invert: false,
        },
    }];
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    assert!(graph.contains("sendcmd=c='0-1 [expr] lumakey@"));
    assert!(graph.contains(" threshold "));
}

fn effect(id: &str, effect: Effect) -> ResolvedEffect {
    ResolvedEffect {
        id: EffectId::new(id).unwrap(),
        active_range: TimeRange::new(time(0), time(600)).unwrap(),
        effect,
    }
}

fn clip(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut veac_plan::ResolvedClip {
    &mut plan.sequences[0].tracks[0].clips[0]
}
