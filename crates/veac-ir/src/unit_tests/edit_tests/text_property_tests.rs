use super::*;

#[path = "text_property_tests/geometry.rs"]
mod geometry;

#[test]
fn typed_text_properties_and_animation_keyframes_are_mutable() {
    let project = sample_project();
    let clip_id = ItemId::new("itm_caption").unwrap();
    let operations = vec![
        text(
            &clip_id,
            TextProperty::Font(FontRef::Family {
                family: "Inter".to_owned(),
            }),
        ),
        text(
            &clip_id,
            TextProperty::FallbackFonts(vec![FontRef::Material {
                material_id: MaterialId::new("med_font").unwrap(),
            }]),
        ),
        text(&clip_id, TextProperty::FontWeight(FontWeight::Bold)),
        text(&clip_id, TextProperty::FontStyle(FontStyle::Italic)),
        text(&clip_id, TextProperty::SizePixels(44.0)),
        text(
            &clip_id,
            TextProperty::Color(Color {
                red: 10,
                green: 20,
                blue: 30,
                alpha: 255,
            }),
        ),
        text(&clip_id, TextProperty::TrackingPixels(2.0)),
        text(&clip_id, TextProperty::LineHeight(1.4)),
        text(
            &clip_id,
            TextProperty::Layout(TextLayout {
                box_width_pixels: Some(300.0),
                box_height_pixels: Some(80.0),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
                horizontal_alignment: HorizontalTextAlignment::Right,
                vertical_alignment: VerticalTextAlignment::Bottom,
                ..TextLayout::default()
            }),
        ),
        text(
            &clip_id,
            TextProperty::Background(Some(TextBackground {
                color: Color {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 128,
                },
                padding_pixels: 4.0,
            })),
        ),
        text(
            &clip_id,
            TextProperty::Outline(Some(TextOutline {
                color: Color {
                    red: 255,
                    green: 255,
                    blue: 255,
                    alpha: 255,
                },
                width_pixels: 2.0,
            })),
        ),
        text(&clip_id, TextProperty::Shadow(None)),
        text(
            &clip_id,
            TextProperty::Spans(vec![TextSpan {
                start: 0,
                end: 2,
                font: None,
                font_weight: Some(FontWeight::Black),
                font_style: None,
                size_pixels: None,
                color: None,
            }]),
        ),
        text(
            &clip_id,
            TextProperty::Animation(Some(TextAnimation {
                granularity: TextGranularity::Whole,
                transform: TextUnitTransform::default(),
                reveal: Animatable::constant(1.0),
                highlight: None,
                opacity: Animatable::Keyframes {
                    keyframes: vec![
                        number_key("kf_text_opacity_start", 0, 0.5),
                        number_key("kf_text_opacity_end", 300, 1.0),
                    ],
                },
                stagger: time(0),
            })),
        ),
        EditOperation::EditKeyframe {
            edit: KeyframeEdit::UpsertNumber {
                clip_id: clip_id.clone(),
                target: NumberCurveTarget::TextOpacity,
                keyframe: number_key("kf_text_opacity_start", 0, 0.7),
            },
        },
        EditOperation::EditKeyframe {
            edit: KeyframeEdit::UpsertNumber {
                clip_id: clip_id.clone(),
                target: NumberCurveTarget::TextReveal,
                keyframe: number_key("kf_text_reveal_edit", 0, 1.0),
            },
        },
    ];
    let outcome = apply_edit_batch(&project, &batch("op_text_properties", &project, operations));
    let updated = applied(outcome);
    let clip = &updated.project.sequences[0].tracks[1].clips[0];
    let ClipSource::Caption { style, .. } = &clip.source else {
        panic!("caption")
    };
    assert_eq!(style.font_weight, FontWeight::Bold);
    assert_eq!(style.fallback_fonts.len(), 1);
    assert_eq!(style.layout.wrap, TextWrap::Word);
    assert_eq!(style.spans.len(), 1);
    assert!(matches!(
        style.animation.as_ref().unwrap().opacity,
        Animatable::Keyframes { .. }
    ));

    let mutation = batch(
        "op_text_keyframe_mutation",
        &updated,
        vec![
            EditOperation::EditKeyframe {
                edit: KeyframeEdit::Move {
                    clip_id: clip_id.clone(),
                    keyframe_id: KeyframeId::new("kf_text_reveal_edit").unwrap(),
                    time: time(10),
                },
            },
            EditOperation::EditKeyframe {
                edit: KeyframeEdit::Remove {
                    clip_id,
                    keyframe_id: KeyframeId::new("kf_text_opacity_start").unwrap(),
                },
            },
        ],
    );
    let mutated = applied(apply_edit_batch(&updated, &mutation));
    let ClipSource::Caption { style, .. } = &mutated.project.sequences[0].tracks[1].clips[0].source
    else {
        panic!("caption")
    };
    assert_eq!(
        style
            .animation
            .as_ref()
            .unwrap()
            .reveal
            .keyframes()
            .unwrap()[0]
            .time,
        time(10)
    );
    assert_eq!(
        style
            .animation
            .as_ref()
            .unwrap()
            .opacity
            .keyframes()
            .unwrap()
            .len(),
        1
    );
}

fn text(id: &ItemId, property: TextProperty) -> EditOperation {
    EditOperation::SetTextProperty {
        clip_id: id.clone(),
        property,
    }
}
