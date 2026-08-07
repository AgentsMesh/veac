use super::*;
use veac_codegen::emitter::CodegenErrorKind;
use veac_plan::{ResolvedEffect, ResolvedTextSpan, ResolvedTextStyle};

mod cases;
mod geometry;
pub(super) mod support;

use cases::*;
use support::*;

#[test]
fn ass_preserves_style_speaker_rich_spans_position_and_stable_deduplication() {
    let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::Ass);
    normalize(&mut plan);
    let (caption, speaker) = first_caption(&mut plan);
    caption.text = "Path \\ {title}\nNext".to_owned();
    caption.styled_mut().unwrap().spans.push(ResolvedTextSpan {
        start: 0,
        end: 4,
        font: None,
        font_weight: Some(FontWeight::Bold),
        font_style: None,
        size_pixels: Some(40.0),
        color: Some(color(255, 0, 0, 255)),
    });
    *speaker = Some("Narrator".to_owned());
    let first = emit_all(&plan, &bindings).unwrap();
    let second = emit_all(&plan, &bindings).unwrap();
    let rendered = content(&first);
    assert_eq!(rendered, content(&second));
    assert_eq!(
        rendered
            .lines()
            .filter(|line| line.starts_with("Style: VEAC"))
            .count(),
        1
    );
    let style = rendered
        .lines()
        .find(|line| line.starts_with("Style: VEAC0001,"))
        .unwrap();
    assert!(style.contains(",32,&H37563412"));
    assert!(!style.contains(",Arial,48,"));
    assert!(rendered.contains(",VEAC0001,Narrator,0,0,0,,"));
    let canvas = &plan
        .sequences
        .iter()
        .find(|sequence| sequence.id == plan.entry_sequence_id)
        .unwrap()
        .settings;
    assert!(rendered.contains(&format!(
        "\\pos({},{})",
        canvas.width + 12,
        canvas.height - 8
    )));
    assert!(rendered.contains(" \\\\ \\{title\\}\\NNext"));
    assert!(rendered.contains("\\fs40\\1c&H0000FF&\\1a&H00&"));
    assert!(rendered.contains("\\blur2\\xshad2\\yshad3"));
    assert!(rendered.contains("Dialogue: 1,0:00:01.00,0:00:02.00"));
}

#[test]
fn ass_rejects_every_unrepresentable_style_family_without_loss() {
    for (code, mutate) in [
        (
            "CAPTION_ASS_FALLBACK_FONTS",
            fallback as fn(&mut ResolvedTextStyle),
        ),
        ("CAPTION_ASS_OBLIQUE", oblique),
        ("CAPTION_ASS_LINE_HEIGHT", line_height),
        ("CAPTION_ASS_TEXT_BOX", text_box),
        ("CAPTION_ASS_WRITING_MODE", writing_mode),
        ("CAPTION_ASS_TEXT_PATH", text_path),
        ("CAPTION_ASS_BACKGROUND", background),
        ("CAPTION_ASS_ANIMATION", animation),
    ] {
        let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::Ass);
        normalize(&mut plan);
        mutate(first_caption(&mut plan).0.styled_mut().unwrap());
        let diagnostic = emit_all(&plan, &bindings).unwrap_err().diagnostics()[0].clone();
        assert_eq!(diagnostic.kind, CodegenErrorKind::UnsupportedCaptionFeature);
        assert_eq!(diagnostic.code, code);
    }
}

#[test]
fn ass_rejects_unsafe_speaker_font_span_visual_and_inexact_time() {
    for (code, mutation) in [
        ("CAPTION_ASS_SPEAKER", 0_u8),
        ("PLAN_TEXT_INVALID", 1),
        ("CAPTION_ASS_SPAN_OBLIQUE", 2),
        ("CAPTION_ASS_VISUAL_UNSUPPORTED", 3),
        ("CAPTION_ASS_SPAN_INVALID", 4),
        ("CAPTION_ASS_VISUAL_UNSUPPORTED", 5),
        ("CAPTION_ASS_VISUAL_UNSUPPORTED", 6),
    ] {
        let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::Ass);
        normalize(&mut plan);
        let (content, speaker) = first_caption(&mut plan);
        match mutation {
            0 => *speaker = Some("Alice,Bob".to_owned()),
            1 => content.styled_mut().unwrap().font.face_index = u32::MAX,
            2 => content
                .styled_mut()
                .unwrap()
                .spans
                .push(span(0, 1, Some(FontStyle::Oblique))),
            3 => {
                let clip = caption_clip(&mut plan);
                clip.visual.as_mut().unwrap().transform.scale =
                    Animatable::constant(Vec2 { x: 2.0, y: 1.0 });
            }
            4 => {
                content.text = "e\u{301}".to_owned();
                content.styled_mut().unwrap().spans.push(span(0, 1, None));
            }
            5 => caption_clip(&mut plan).effects.push(ResolvedEffect {
                id: EffectId::new("fx_caption_blur").unwrap(),
                active_range: TimeRange::new(time(0), time(600)).unwrap(),
                effect: Effect::VideoBlur {
                    radius: Animatable::constant(2.0),
                },
            }),
            6 => {
                caption_clip(&mut plan)
                    .visual
                    .as_mut()
                    .unwrap()
                    .transform
                    .shear = Vec2 { x: 0.25, y: 0.0 };
            }
            _ => unreachable!(),
        }
        let diagnostic = emit_all(&plan, &bindings).unwrap_err().diagnostics()[0].clone();
        assert_eq!(diagnostic.code, code);
    }

    let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::Ass);
    normalize(&mut plan);
    plan.sequences[0]
        .tracks
        .iter_mut()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap()
        .clips
        .first_mut()
        .unwrap()
        .record_range
        .start = RationalTime::new(301, 600).unwrap();
    assert_eq!(
        emit_all(&plan, &bindings).unwrap_err().diagnostics()[0].code,
        "PLAN_CAPTION_SIDECAR_INVALID"
    );
}
