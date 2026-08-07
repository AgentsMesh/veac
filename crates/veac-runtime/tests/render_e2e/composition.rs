use tempfile::tempdir;

use super::support::*;

#[test]
fn keyframes_mask_blend_and_effect_change_rendered_pixels() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    let mut layer = solid_clip("itm_layer", color(255, 0, 0), 0, 2_000);
    layer.visual = Some(animated_layer());
    layer.effects.push(brightness_effect());
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_background",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_background", color(0, 0, 255), 0, 2_000)],
        ),
        track("trk_layer", TrackKind::Visual, 1, vec![layer]),
    ]);
    let output = temp.path().join("composition.mp4");
    render(canonical, &BTreeMap::new(), &output);

    assert_media_contract(&output, 0, 2.0);
    let early_center = rgb_at(&output, 0.25, 33, 27);
    let early_future = rgb_at(&output, 0.25, 75, 27);
    let early_corner = rgb_at(&output, 0.25, 20, 14);
    let late_past = rgb_at(&output, 1.75, 21, 27);
    let late_center = rgb_at(&output, 1.75, 63, 27);
    assert_composited(early_center);
    assert_blue(early_future);
    assert!(
        early_center[0] > early_corner[0].saturating_add(100) && early_corner[2] > 180,
        "mask center={early_center:?}, corner={early_corner:?}"
    );
    assert_blue(late_past);
    assert_composited(late_center);
}

fn animated_layer() -> VisualProperties {
    let position = Animatable::Keyframes {
        keyframes: vec![
            Keyframe {
                id: KeyframeId::new("kf_position_start").unwrap(),
                time: time(0),
                value: point(-20.0, 0.0),
                interpolation: Interpolation::Linear,
            },
            Keyframe {
                id: KeyframeId::new("kf_position_end").unwrap(),
                time: time(2_000),
                value: point(20.0, 0.0),
                interpolation: Interpolation::Linear,
            },
        ],
    };
    VisualProperties {
        placement: Placement::Anchor {
            anchor: Anchor::Center,
            inset: Vec2 { x: 0.0, y: 0.0 },
        },
        frame: Some(Frame {
            width: pixels(32.0),
            height: pixels(32.0),
            fit: FitMode::Fill,
        }),
        transform: Transform2D {
            position,
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            shear: Vec2 { x: 0.0, y: 0.0 },
            flip_horizontal: false,
            flip_vertical: false,
            rotation_degrees: Animatable::constant(0.0),
            anchor: Vec2 { x: 0.5, y: 0.5 },
            crop: None,
        },
        opacity: Animatable::constant(1.0),
        compositing: Compositing {
            z_index: 1,
            blend_mode: BlendMode::Screen,
        },
        masks: vec![Mask {
            shape: MaskShape::Circle,
            position: Animatable::constant(Vec2 { x: 0.5, y: 0.5 }),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            rotation_degrees: Animatable::constant(0.0),
            feather_pixels: Animatable::constant(0.0),
            expansion_pixels: Animatable::constant(0.0),
            invert: false,
        }],
        card: None,
        color_pipeline: None,
    }
}

fn brightness_effect() -> EffectInstance {
    EffectInstance {
        id: EffectId::new("fx_brightness").unwrap(),
        enabled: true,
        enable_range: None,
        effect: Effect::VideoColorAdjust {
            brightness: Animatable::constant(0.2),
            contrast: Animatable::constant(1.0),
            saturation: Animatable::constant(1.0),
        },
    }
}

fn point(x: f64, y: f64) -> Point {
    Point {
        x: pixels(x),
        y: pixels(y),
    }
}

fn pixels(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}

fn assert_blue(pixel: [u8; 3]) {
    assert!(pixel[2] > 180 && pixel[0] < 40, "pixel={pixel:?}");
}

fn assert_composited(pixel: [u8; 3]) {
    assert!(
        pixel[0] > 180 && pixel[1] > 20 && pixel[2] > 180,
        "pixel={pixel:?}"
    );
}
