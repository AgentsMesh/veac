use veac_plan::canonical::*;

use super::{clip, effect, graph};
use crate::unit_tests::emitter_tests::support::{fixture, resolved, time};

#[test]
fn every_curve_capable_video_effect_has_a_frame_evaluated_backend_path() {
    for (kind, parameter, value, marker) in [
        ("video.blur", "radius", 4.0, "gblur@"),
        ("video.sharpen", "amount", 3.0, "cas@"),
        ("video.vignette", "amount", 0.8, "eval=frame"),
        ("video.grain", "amount", 0.7, "blend=all_expr"),
        ("video.chroma_key", "similarity", 0.5, "colorkey@"),
        ("video.luma_key", "threshold", 0.5, "lumakey@"),
        ("video.chroma_spill", "amount", 0.8, "despill@"),
    ] {
        let mut plan = resolved(&fixture());
        clip(&mut plan).effects = vec![effect("fx_animated", kind, [(parameter, animated(value))])];
        let rendered = graph(&plan);
        assert!(rendered.contains(marker), "missing {marker}: {rendered}");
        if !matches!(kind, "video.vignette" | "video.grain") {
            assert!(rendered.contains("sendcmd=c='0-1 [expr]"), "{rendered}");
        }
    }
}

#[test]
fn cubic_runtime_commands_escape_nested_command_delimiters() {
    let mut plan = resolved(&fixture());
    let mut value = animated(0.6);
    let ParameterValue::NumberCurve {
        value: Animatable::Keyframes { keyframes },
    } = &mut value
    else {
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
        "video.chroma_key",
        [("similarity", value)],
    )];
    let rendered = graph(&plan);
    assert!(rendered.contains("st(1\\\\,"), "{rendered}");
    assert!(rendered.contains("\\\\;"), "{rendered}");
}

fn animated(end: f64) -> ParameterValue {
    ParameterValue::NumberCurve {
        value: Animatable::Keyframes {
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
        },
    }
}
