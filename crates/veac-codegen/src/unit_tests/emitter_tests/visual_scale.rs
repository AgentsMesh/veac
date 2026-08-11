use veac_plan::canonical::*;

use super::support::{bindings, emit_video_command, fixture, resolved, time};

#[test]
fn growing_animated_scale_uses_a_fixed_extent_before_rotation() {
    let mut plan = resolved(&fixture());
    let visual = plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap();
    visual.frame = Some(Frame {
        width: pixels(320.0),
        height: pixels(180.0),
        fit: FitMode::Fill,
    });
    visual.transform.anchor = Vec2 { x: 0.5, y: 0.0 };
    visual.transform.scale = Animatable::Keyframes {
        keyframes: vec![scale_key("edge", 0, 0.08), scale_key("face", 300, 1.0)],
    };
    visual.transform.rotation_degrees = Animatable::constant(7.0);
    let graph = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .filter_graph
        .unwrap();
    let stable = "pad=w=320:h=180:x='0.5*320-0.5*iw':y='0*180-0*ih'";
    assert!(graph.contains(stable), "{graph}");
    let fixed = graph.find("scaleextentv").unwrap();
    let pivot = graph.find("pivotv").unwrap();
    assert!(fixed < pivot, "{graph}");
}

fn scale_key(id: &str, milliseconds: i64, y: f64) -> Keyframe<Vec2> {
    Keyframe {
        id: KeyframeId::new(format!("kf_scale_{id}")).unwrap(),
        time: time(milliseconds),
        value: Vec2 { x: 1.0, y },
        interpolation: Interpolation::Linear,
    }
}

fn pixels(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}
