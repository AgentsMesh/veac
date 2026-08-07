use super::super::*;
use veac_plan::{ResolvedText, ResolvedTextSpan};

pub(crate) fn normalize(plan: &mut ResolvedRenderPlan) {
    let clips = &mut plan.sequences[0]
        .tracks
        .iter_mut()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap()
        .clips;
    for clip in clips {
        let ResolvedClipSource::Caption { content, .. } = &mut clip.source else {
            continue;
        };
        content.styled_mut().unwrap().fallback_fonts.clear();
        content.styled_mut().unwrap().font_weight = FontWeight::Medium;
        content.styled_mut().unwrap().font_style = FontStyle::Italic;
        content.styled_mut().unwrap().size_pixels = 32.0;
        content.styled_mut().unwrap().color = color(0x12, 0x34, 0x56, 200);
        content.styled_mut().unwrap().tracking_pixels = 1.25;
        content.styled_mut().unwrap().line_height = 1.0;
        content.styled_mut().unwrap().layout = TextLayout {
            horizontal_alignment: HorizontalTextAlignment::Right,
            vertical_alignment: VerticalTextAlignment::Bottom,
            ..TextLayout::default()
        };
        content.styled_mut().unwrap().path = None;
        content.styled_mut().unwrap().background = None;
        content.styled_mut().unwrap().animation = None;
        content.styled_mut().unwrap().spans.clear();
        clip.effects.clear();
        normalize_visual(clip.visual.as_mut().unwrap());
    }
}

fn normalize_visual(visual: &mut veac_plan::EffectiveVisualProperties) {
    visual.placement = Placement::Anchor {
        anchor: Anchor::Center,
        inset: Vec2 { x: 0.0, y: 0.0 },
    };
    visual.frame = None;
    visual.transform.position = Animatable::constant(Point {
        x: pixels(12.0),
        y: pixels(-8.0),
    });
    visual.transform.scale = Animatable::constant(Vec2 { x: 1.0, y: 1.0 });
    visual.transform.shear = Vec2 { x: 0.0, y: 0.0 };
    visual.transform.rotation_degrees = Animatable::constant(0.0);
    visual.transform.anchor = Vec2 { x: 0.5, y: 0.5 };
    visual.transform.flip_horizontal = false;
    visual.transform.flip_vertical = false;
    visual.transform.crop = None;
    visual.opacity = Animatable::constant(1.0);
    visual.compositing.blend_mode = BlendMode::Normal;
    visual.masks.clear();
    visual.track_matte = None;
    visual.card = None;
    visual.color_pipeline = None;
}

pub(super) fn first_caption(
    plan: &mut ResolvedRenderPlan,
) -> (&mut ResolvedText, &mut Option<String>) {
    let ResolvedClipSource::Caption {
        content, speaker, ..
    } = &mut caption_clip(plan).source
    else {
        unreachable!()
    };
    (content, speaker)
}

pub(super) fn caption_clip(plan: &mut ResolvedRenderPlan) -> &mut veac_plan::ResolvedClip {
    plan.sequences[0]
        .tracks
        .iter_mut()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap()
        .clips
        .iter_mut()
        .find(|clip| matches!(clip.source, ResolvedClipSource::Caption { .. }))
        .unwrap()
}

pub(super) fn color(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
    Color {
        red,
        green,
        blue,
        alpha,
    }
}

pub(super) fn span(start: u32, end: u32, font_style: Option<FontStyle>) -> ResolvedTextSpan {
    ResolvedTextSpan {
        start,
        end,
        font: None,
        font_weight: None,
        font_style,
        size_pixels: None,
        color: None,
    }
}

fn pixels(value: f64) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}
