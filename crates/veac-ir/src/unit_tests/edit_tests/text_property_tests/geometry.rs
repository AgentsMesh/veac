use super::*;

#[test]
fn text_geometry_and_transform_curves_have_typed_edit_targets() {
    let project = sample_project();
    let clip_id = ItemId::new("itm_caption").unwrap();
    let operations = vec![
        text(
            &clip_id,
            TextProperty::WritingMode(TextWritingMode::VerticalRl),
        ),
        text(
            &clip_id,
            TextProperty::Orientation(TextOrientation::Upright),
        ),
        text(
            &clip_id,
            TextProperty::Animation(Some(TextAnimation {
                granularity: TextGranularity::Word,
                transform: TextUnitTransform::default(),
                reveal: Animatable::constant(1.0),
                highlight: None,
                opacity: Animatable::constant(1.0),
                stagger: time(0),
            })),
        ),
        keyframe(KeyframeEdit::UpsertPoint {
            clip_id: clip_id.clone(),
            target: PointCurveTarget::TextPositionOffset,
            keyframe: point_key("kf_text_position", 0),
        }),
        keyframe(KeyframeEdit::UpsertVec2 {
            clip_id: clip_id.clone(),
            target: Vec2CurveTarget::TextScale,
            keyframe: vec2_key("kf_text_scale", 0, 1.2),
        }),
        keyframe(KeyframeEdit::UpsertNumber {
            clip_id: clip_id.clone(),
            target: NumberCurveTarget::TextRotationDegrees,
            keyframe: number_key("kf_text_rotation", 0, 12.0),
        }),
    ];
    let updated = applied(apply_edit_batch(
        &project,
        &batch("op_text_geometry", &project, operations),
    ));
    let style = caption_style(&updated);
    assert_eq!(style.layout.writing_mode, TextWritingMode::VerticalRl);
    assert_eq!(style.layout.orientation, TextOrientation::Upright);
    let transform = &style.animation.as_ref().unwrap().transform;
    assert!(transform.position_offset.keyframes().is_some());
    assert!(transform.scale.keyframes().is_some());
    assert!(transform.rotation_degrees.keyframes().is_some());

    let path = TextPath {
        points: vec![point(0.0, 20.0), point(200.0, 20.0)],
        start_offset: pixels(100.0),
        reverse: false,
        alignment: TextPathAlignment::Center,
    };
    let path_edit = batch(
        "op_text_path",
        &updated,
        vec![
            text(
                &clip_id,
                TextProperty::WritingMode(TextWritingMode::HorizontalTb),
            ),
            text(&clip_id, TextProperty::Path(Some(path.clone()))),
        ],
    );
    let updated = applied(apply_edit_batch(&updated, &path_edit));
    assert_eq!(caption_style(&updated).path.as_ref(), Some(&path));
}

fn keyframe(edit: KeyframeEdit) -> EditOperation {
    EditOperation::EditKeyframe { edit }
}

fn caption_style(project: &ProjectEnvelope) -> &TextStyle {
    match &project.project.sequences[0].tracks[1].clips[0].source {
        ClipSource::Caption { style, .. } => style,
        _ => panic!("caption fixture"),
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
