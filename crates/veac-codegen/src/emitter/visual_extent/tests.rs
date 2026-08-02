use super::*;

#[test]
fn source_extent_covers_media_nested_and_generated() {
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
            plan.sequences[0].settings.height,
        ))
    );

    nested.source = ResolvedClipSource::Generated {
        generator: Generator::Transparent,
    };
    assert_eq!(source_extent(&plan, &nested), None);
}

#[test]
fn placement_extent_uses_spring_extrema() {
    let mut plan = crate::unit_tests::emitter_tests::support::resolved(
        &crate::unit_tests::emitter_tests::support::fixture(),
    );
    let sequence = &mut plan.sequences[0];
    let clip = &mut sequence.tracks[0].clips[0];
    let visual = clip.visual.as_mut().unwrap();
    visual.transform.scale = Animatable::Keyframes {
        keyframes: vec![
            key(
                "kf_extent_a",
                10.0,
                Interpolation::Spring {
                    frequency: 1.5,
                    decay: 6.0,
                    initial_velocity: 0.0,
                },
            ),
            key("kf_extent_b", 11.0, Interpolation::Linear),
        ],
    };
    let sequence = &plan.sequences[0];
    let clip = &sequence.tracks[0].clips[0];
    let extent = placement_extent(&plan, sequence, clip, clip.visual.as_ref().unwrap()).unwrap();
    assert!(extent.0 > u128::from(sequence.settings.width) * 11);
}

#[test]
fn generated_identity_keeps_placement_and_transform_anchor_contracts() {
    let mut plan = crate::unit_tests::emitter_tests::support::resolved(
        &crate::unit_tests::emitter_tests::support::fixture(),
    );
    let clip = &mut plan.sequences[0].tracks[0].clips[0];
    clip.source = ResolvedClipSource::Generated {
        generator: Generator::Transparent,
    };
    clip.effects.clear();
    {
        let visual = clip.visual.as_mut().unwrap();
        visual.frame = None;
        visual.card = None;
        visual.transform.scale = Animatable::constant(Vec2 { x: 1.0, y: 1.0 });
        visual.transform.rotation_degrees = Animatable::constant(0.0);
        visual.transform.shear = Vec2 { x: 0.0, y: 0.0 };
        visual.transform.crop = None;
        visual.transform.anchor = Vec2 { x: 0.5, y: 0.5 };
    }
    assert!(!requires_canvas_placement(
        clip,
        clip.visual.as_ref().unwrap()
    ));

    clip.visual.as_mut().unwrap().placement = Placement::Anchor {
        anchor: Anchor::TopLeft,
        inset: Vec2 { x: 0.0, y: 0.0 },
    };
    assert!(requires_canvas_placement(
        clip,
        clip.visual.as_ref().unwrap()
    ));

    {
        let visual = clip.visual.as_mut().unwrap();
        visual.placement = Placement::Anchor {
            anchor: Anchor::Center,
            inset: Vec2 { x: 0.0, y: 0.0 },
        };
        visual.transform.anchor = Vec2 { x: 0.0, y: 0.0 };
    }
    assert!(requires_canvas_placement(
        clip,
        clip.visual.as_ref().unwrap()
    ));
}

fn key(id: &str, value: f64, interpolation: Interpolation) -> Keyframe<Vec2> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: RationalTime::zero(600).unwrap(),
        value: Vec2 { x: value, y: value },
        interpolation,
    }
}
