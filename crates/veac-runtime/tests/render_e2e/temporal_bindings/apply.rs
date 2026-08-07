use tempfile::tempdir;

use super::super::support::*;
use super::{install_sequence_time, sequence_curve};

#[test]
fn sequence_time_binding_drives_apply_mix_opacity() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("temporal-apply-opacity.mp4");
    let mut project = scene();
    let binding = install_sequence_time(
        &mut project,
        "apply_opacity",
        TemporalType::Scalar,
        sequence_curve(scalar(0.0), scalar(1.0)),
        1,
    );
    let mut adjustment = brightness_apply("apl_temporal_opacity", 0.5);
    adjustment.mix.opacity = Animatable::Binding {
        binding_id: binding,
    };
    project.project.sequences[0].applies.push(adjustment);
    render(project, &BTreeMap::new(), &output);

    let early = luma(rgb_at(&output, 0.1, WIDTH / 2, HEIGHT / 2));
    let late = luma(rgb_at(&output, 0.8, WIDTH / 2, HEIGHT / 2));
    assert!(late > early + 35, "apply opacity luma {early}..{late}");
}

#[test]
fn sequence_time_binding_drives_apply_effect_parameter() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("temporal-apply-effect.mp4");
    let mut project = scene();
    let binding = install_sequence_time(
        &mut project,
        "apply_effect",
        TemporalType::Scalar,
        sequence_curve(scalar(0.0), scalar(0.5)),
        1,
    );
    let stage = effect_stage(
        "aps_temporal_effect",
        Effect::VideoColorAdjust {
            brightness: Animatable::Binding {
                binding_id: binding,
            },
            contrast: Animatable::constant(1.0),
            saturation: Animatable::constant(1.0),
        },
    );
    project.project.sequences[0].applies.push(apply(
        "apl_temporal_effect",
        layer_target(),
        0,
        1_000,
        vec![stage],
    ));
    render(project, &BTreeMap::new(), &output);

    let early = luma(rgb_at(&output, 0.1, WIDTH / 2, HEIGHT / 2));
    let late = luma(rgb_at(&output, 0.8, WIDTH / 2, HEIGHT / 2));
    assert!(late > early + 65, "apply effect luma {early}..{late}");
}

#[test]
fn sequence_time_binding_drives_an_apply_owned_mask() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("temporal-apply-mask.mp4");
    let mut project = scene();
    let binding = install_sequence_time(
        &mut project,
        "apply_mask_position",
        TemporalType::Vec2,
        sequence_curve(vector(0.25, 0.5), vector(0.75, 0.5)),
        1,
    );
    let mut adjustment = brightness_apply("apl_temporal_mask", 0.5);
    let mut mask = default_mask(MaskShape::Circle);
    mask.position = Animatable::Binding {
        binding_id: binding,
    };
    mask.scale = Animatable::constant(Vec2 { x: 0.3, y: 0.5 });
    adjustment.mix.masks.push(mask);
    project.project.sequences[0].applies.push(adjustment);
    render(project, &BTreeMap::new(), &output);

    let early_left = luma(rgb_at(&output, 0.1, 29, HEIGHT / 2));
    let early_right = luma(rgb_at(&output, 0.1, 67, HEIGHT / 2));
    let late_left = luma(rgb_at(&output, 0.8, 29, HEIGHT / 2));
    let late_right = luma(rgb_at(&output, 0.8, 67, HEIGHT / 2));
    assert!(
        early_left > early_right + 45,
        "early {early_left}/{early_right}"
    );
    assert!(late_right > late_left + 45, "late {late_left}/{late_right}");
}

fn scene() -> ProjectEnvelope {
    let mut project = project(false);
    project.project.sequences[0].tracks.push(track(
        "trk_temporal_apply",
        TrackKind::Video,
        0,
        vec![solid_clip(
            "itm_temporal_apply",
            color(28, 32, 36),
            0,
            1_000,
        )],
    ));
    project
}

fn brightness_apply(id: &str, brightness: f64) -> Apply {
    apply(
        id,
        layer_target(),
        0,
        1_000,
        vec![brightness_stage(&format!("aps_{id}"), brightness)],
    )
}

fn layer_target() -> ApplyTarget {
    ApplyTarget::Layer {
        track_id: TrackId::new("trk_temporal_apply").unwrap(),
    }
}

fn scalar(value: f64) -> TemporalValue {
    TemporalValue::Scalar { value }
}

fn vector(x: f64, y: f64) -> TemporalValue {
    TemporalValue::Vec2 {
        value: Vec2 { x, y },
    }
}

fn luma(value: [u8; 3]) -> u16 {
    value.into_iter().map(u16::from).sum::<u16>() / 3
}
