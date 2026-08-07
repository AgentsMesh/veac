use std::collections::BTreeSet;

use veac_ir::{Animatable, Interpolation, TextGranularity};

use super::{clips, envelope, style};

#[test]
fn all_granularities_interpolations_and_scoped_key_ids_are_observable() {
    let first = envelope();
    let second = envelope();
    let first_clips = clips(&first);
    let granularities = first_clips
        .iter()
        .map(|clip| style(&clip.source).animation.as_ref().unwrap().granularity)
        .collect::<Vec<_>>();
    assert_eq!(
        granularities,
        [
            TextGranularity::Word,
            TextGranularity::Whole,
            TextGranularity::Grapheme,
            TextGranularity::Line
        ]
    );
    let animation = style(&first_clips[0].source).animation.as_ref().unwrap();
    let keys = animation.reveal.keyframes().unwrap();
    assert_eq!(
        keys.iter().map(|key| key.value).collect::<Vec<_>>(),
        vec![0.0, 0.15, 0.3, 0.45, 0.6, 0.6, 0.8, 1.0]
    );
    assert!(matches!(keys[0].interpolation, Interpolation::Hold));
    assert!(matches!(keys[1].interpolation, Interpolation::Linear));
    assert!(matches!(keys[2].interpolation, Interpolation::EaseIn));
    assert!(matches!(keys[3].interpolation, Interpolation::EaseOut));
    assert!(matches!(
        keys[4].interpolation,
        Interpolation::Spring { .. }
    ));
    assert!(
        matches!(keys[5].interpolation, Interpolation::CubicBezier { y1, y2, .. } if y1 == -0.4 && y2 == 1.4)
    );
    assert!(matches!(keys[6].interpolation, Interpolation::EaseInOut));

    let ids = |value: &veac_ir::ProjectEnvelope| {
        clips(value)
            .iter()
            .flat_map(|clip| {
                let animation = style(&clip.source).animation.as_ref().unwrap();
                let mut ids = animation
                    .transform
                    .position_offset
                    .keyframes()
                    .unwrap()
                    .iter()
                    .map(|key| key.id.to_string())
                    .collect::<Vec<_>>();
                ids.extend(
                    animation
                        .reveal
                        .keyframes()
                        .unwrap()
                        .iter()
                        .map(|key| key.id.to_string()),
                );
                ids
            })
            .collect::<Vec<_>>()
    };
    let first_ids = ids(&first);
    assert_eq!(first_ids, ids(&second));
    assert_eq!(
        first_ids.len(),
        first_ids.iter().collect::<BTreeSet<_>>().len()
    );
    assert!(matches!(
        animation.opacity,
        Animatable::Constant { value: 1.0 }
    ));
}
