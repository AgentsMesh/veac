use veac_ir::{Animatable, Effect, Interpolation};

use crate::support::{authored_key, sequence_by_key};

#[test]
fn directional_blur_has_two_axes_and_an_animated_radius() {
    let envelope = crate::support::lower_example("video-effects/main.veac");
    let sequence = sequence_by_key(&envelope, "main");
    let clips = sequence
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .collect::<Vec<_>>();
    let horizontal = effect(&clips, "directional-horizontal");
    let vertical = effect(&clips, "directional-vertical");
    let animated = effect(&clips, "directional-animated");

    assert_direction(horizontal, 0.0, &Animatable::constant(28.0));
    assert_direction(vertical, 90.0, &Animatable::constant(28.0));
    let Effect::VideoDirectionalBlur {
        angle_degrees,
        radius: Animatable::Keyframes { keyframes },
    } = animated
    else {
        panic!("animated segment must use directional blur keyframes");
    };
    assert_eq!(angle_degrees, &Animatable::constant(45.0));
    assert_eq!(
        keyframes
            .iter()
            .map(|key| (key.time.value, key.value, key.interpolation.clone()))
            .collect::<Vec<_>>(),
        vec![
            (0, 0.0, Interpolation::EaseInOut),
            (600, 36.0, Interpolation::EaseInOut),
            (1_200, 0.0, Interpolation::Linear),
        ]
    );
}

fn effect<'a>(clips: &[&'a veac_ir::Clip], key: &str) -> &'a Effect {
    let clip = clips
        .iter()
        .find(|clip| authored_key(&clip.authorship) == Some(key))
        .unwrap_or_else(|| panic!("missing {key}"));
    assert_eq!(clip.effects.len(), 1, "{key} needs one attributable effect");
    &clip.effects[0].effect
}

fn assert_direction(effect: &Effect, angle: f64, radius: &Animatable<f64>) {
    let Effect::VideoDirectionalBlur {
        angle_degrees,
        radius: actual_radius,
    } = effect
    else {
        panic!("segment must use directional blur");
    };
    assert_eq!(angle_degrees, &Animatable::constant(angle));
    assert_eq!(actual_radius, radius);
}
