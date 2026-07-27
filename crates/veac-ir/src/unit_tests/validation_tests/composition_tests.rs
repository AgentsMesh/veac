use super::*;

#[test]
fn animated_mask_values_and_closed_paths_fail_closed() {
    let mut project = sample_project();
    let mask = &mut project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .masks[0];
    mask.shape = MaskShape::Path {
        points: vec![
            Vec2 { x: 0.1, y: 0.1 },
            Vec2 { x: 0.5, y: 0.5 },
            Vec2 { x: 0.9, y: 0.9 },
        ],
    };
    mask.position = Animatable::constant(Vec2 { x: 1.1, y: 0.5 });
    mask.scale = Animatable::constant(Vec2 { x: 0.0, y: 1.0 });
    mask.rotation_degrees = Animatable::constant(f64::NAN);
    mask.expansion_pixels = Animatable::constant(f64::INFINITY);
    let codes = validation_codes(&project);
    assert_code(&codes, "MASK_PATH");
    assert_code(&codes, "ANIMATION_VALUE");
}

#[test]
fn matte_reference_type_range_and_cycle_are_validated() {
    let valid = project_with_matte();
    validate(&valid).unwrap();

    let mut missing = valid.clone();
    let RelationKind::Matte { producer, .. } = &mut missing.project.relations[0].kind else {
        unreachable!()
    };
    *producer = RelationEndpoint::item(ItemId::new("itm_absent").unwrap());
    assert_code(&validation_codes(&missing), "RELATION_ENDPOINT_NOT_FOUND");

    let mut short = valid.clone();
    matte(&mut short).record_range.duration = crate::test_support::time(300);
    assert_code(&validation_codes(&short), "MATTE_SOURCE_RANGE");

    let mut typed = valid.clone();
    typed.project.sequences[0].tracks[2].kind = TrackKind::Audio;
    assert_code(&validation_codes(&typed), "MATTE_SOURCE_TYPE");

    let mut cycle = valid;
    cycle.project.relations.push(Relation {
        id: RelationId::new("rel_matte_cycle").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Matte {
            producer: RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
            consumer: RelationEndpoint::item(ItemId::new("itm_matte").unwrap()),
            parameters: MatteRelationParameters {
                mode: TrackMatteMode::Luma,
                invert: true,
            },
        },
    });
    assert_code(&validation_codes(&cycle), "MATTE_CYCLE");
}

fn project_with_matte() -> ProjectEnvelope {
    let mut project = sample_project();
    let mut source = project.project.sequences[0].tracks[0].clips[0].clone();
    source.id = ItemId::new("itm_matte").unwrap();
    source.audio = None;
    source.effects.clear();
    let visual = source.visual.as_mut().unwrap();
    visual.transform.position = Animatable::constant(Point {
        x: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
        y: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
    });
    visual.opacity = Animatable::constant(1.0);
    visual.masks.clear();
    visual.card = None;
    project.project.sequences[0].tracks.push(Track {
        id: TrackId::new("trk_matte").unwrap(),
        kind: TrackKind::Visual,
        order: 1,
        placement_mode: PlacementMode::Free,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips: vec![source],
    });
    project.project.relations.push(Relation {
        id: RelationId::new("rel_matte").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Matte {
            producer: RelationEndpoint::item(ItemId::new("itm_matte").unwrap()),
            consumer: RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
            parameters: MatteRelationParameters {
                mode: TrackMatteMode::Alpha,
                invert: false,
            },
        },
    });
    project
}

fn matte(project: &mut ProjectEnvelope) -> &mut Clip {
    &mut project.project.sequences[0].tracks[2].clips[0]
}
