use super::{mechanism_helpers::text_clip, support::*};
use crate::{canonical::*, plan_hash, resolve, ResolvedClipSource};

#[test]
fn text_fonts_layout_spans_and_animation_resolve_as_owned_plan_data() {
    let mut value = project();
    value.project.materials.extend([
        font_material("med_font"),
        font_material("med_fallback"),
        font_material("med_span"),
    ]);
    let mut clip = text_clip(false);
    let ClipSource::Text { style, .. } = &mut clip.source else {
        panic!("text")
    };
    style.fallback_fonts.push(FontRef::Material {
        material_id: MaterialId::new("med_fallback").unwrap(),
    });
    style.font_weight = FontWeight::Bold;
    style.font_style = FontStyle::Italic;
    style.tracking_pixels = 2.0;
    style.line_height = 1.3;
    style.layout = TextLayout {
        box_width_pixels: Some(400.0),
        box_height_pixels: Some(120.0),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        horizontal_alignment: HorizontalTextAlignment::Right,
        vertical_alignment: VerticalTextAlignment::Bottom,
        ..TextLayout::default()
    };
    style.spans.push(TextSpan {
        start: 0,
        end: 2,
        font: Some(FontRef::Material {
            material_id: MaterialId::new("med_span").unwrap(),
        }),
        font_weight: Some(FontWeight::Black),
        font_style: None,
        size_pixels: Some(50.0),
        color: None,
    });
    style.animation = Some(TextAnimation {
        granularity: TextGranularity::Whole,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        opacity: Animatable::constant(0.5),
        stagger: time(0),
        highlight: None,
    });
    value.project.sequences[0].tracks.push(track(
        "trk_text_advanced",
        TrackKind::Visual,
        10,
        vec![clip],
    ));

    let plan = resolve(&value, None).unwrap().remove(0);
    assert_eq!(plan.inputs.len(), 4);
    let ResolvedClipSource::Text { content } = &plan.sequences[0].tracks[1].clips[0].source else {
        panic!("resolved text")
    };
    let style = content.styled().unwrap();
    assert_eq!(style.fallback_fonts[0].input_id.as_str(), "pin_fallback");
    assert_eq!(
        style.spans[0].font.as_ref().unwrap().input_id.as_str(),
        "pin_span"
    );
    assert_eq!(style.layout.box_width_pixels, Some(400.0));
    assert_eq!(
        style.animation.as_ref().unwrap().opacity,
        Animatable::constant(0.5)
    );
    let json = crate::canonical_plan_json(&plan).unwrap();
    assert_eq!(
        serde_json::from_str::<crate::ResolvedRenderPlan>(&json).unwrap(),
        plan
    );
}

#[test]
fn generated_graphics_survive_resolution_and_change_plan_hash() {
    let mut value = project();
    value.project.materials.clear();
    let clip = &mut value.project.sequences[0].tracks[0].clips[0];
    clip.source_mapping = None;
    clip.source = ClipSource::Generated {
        generator: Generator::Gradient {
            gradient: linear(255),
        },
    };
    let first = resolve(&value, None).unwrap().remove(0);
    let first_hash = plan_hash(&first).unwrap();
    let ClipSource::Generated { generator } =
        &mut value.project.sequences[0].tracks[0].clips[0].source
    else {
        panic!()
    };
    *generator = Generator::Gradient {
        gradient: linear(128),
    };
    let second = resolve(&value, None).unwrap().remove(0);
    assert_ne!(first_hash, plan_hash(&second).unwrap());
    assert!(matches!(
        first.sequences[0].tracks[0].clips[0].source,
        ResolvedClipSource::Generated { .. }
    ));
}

fn linear(alpha: u8) -> Gradient {
    Gradient::Linear {
        start: Vec2 { x: 0.0, y: 0.0 },
        end: Vec2 { x: 1.0, y: 0.0 },
        stops: vec![
            GradientStop {
                offset: 0.0,
                color: Color {
                    red: 255,
                    green: 0,
                    blue: 0,
                    alpha,
                },
            },
            GradientStop {
                offset: 1.0,
                color: Color {
                    red: 0,
                    green: 0,
                    blue: 255,
                    alpha,
                },
            },
        ],
    }
}
