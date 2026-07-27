use super::*;

#[test]
fn source_extent_covers_media_nested_multicam_text_and_generated() {
    let plan = crate::unit_tests::emitter_tests::support::resolved(
        &crate::unit_tests::emitter_tests::support::fixture(),
    );
    let clip = &plan.sequences[0].tracks[0].clips[0];
    assert!(source_extent(&plan, clip).is_some());

    let mut nested = clip.clone();
    nested.source = ResolvedClipSource::Sequence {
        sequence_id: plan.sequences[0].id.clone(),
    };
    assert_eq!(
        source_extent(&plan, &nested),
        Some((
            plan.sequences[0].settings.width,
            plan.sequences[0].settings.height
        ))
    );

    nested.source = ResolvedClipSource::Generated {
        generator: Generator::Transparent,
    };
    assert_eq!(source_extent(&plan, &nested), None);
}

#[test]
fn malformed_and_animated_scales_fail_closed_or_use_axis_maxima() {
    assert_eq!(
        max_scale(&Animatable::constant(Vec2 { x: 0.0, y: 1.0 })),
        None
    );
    let curve = Animatable::Keyframes {
        keyframes: vec![
            key("kf_budget_scale_a", 2.0, 3.0),
            key("kf_budget_scale_b", 4.0, 1.0),
        ],
    };
    assert_eq!(max_scale(&curve), Some((4.0, 3.0)));
}

fn key(id: &str, x: f64, y: f64) -> Keyframe<Vec2> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: RationalTime::zero(600).unwrap(),
        value: Vec2 { x, y },
        interpolation: Interpolation::Linear,
    }
}
