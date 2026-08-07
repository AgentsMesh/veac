use super::*;
use veac_codegen::emitter::CodegenErrorKind;
use veac_plan::ResolvedTextSpan;

#[test]
fn webvtt_preserves_speaker_and_escapes_markup() {
    let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::WebVtt);
    let (caption, speaker) = first(&mut plan);
    caption.text = "<First & line>".to_owned();
    *speaker = Some("A&B".to_owned());
    let output = content(&emit_all(&plan, &bindings).unwrap());
    assert!(output.contains("<v A&amp;B>&lt;First &amp; line&gt;</v>"));
}

#[test]
fn srt_speaker_and_plain_format_rich_spans_are_typed_rejections() {
    let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::Srt);
    *first(&mut plan).1 = Some("Narrator".to_owned());
    let diagnostic = emit_all(&plan, &bindings).unwrap_err().diagnostics()[0].clone();
    assert_eq!(diagnostic.kind, CodegenErrorKind::UnsupportedCaptionFeature);
    assert_eq!(diagnostic.code, "CAPTION_SRT_SPEAKER_UNSUPPORTED");

    for format in [CaptionSidecarFormat::Srt, CaptionSidecarFormat::WebVtt] {
        let (mut plan, bindings, _) = caption_plan(format);
        first(&mut plan)
            .0
            .styled_mut()
            .unwrap()
            .spans
            .push(ResolvedTextSpan {
                start: 0,
                end: 1,
                font: None,
                font_weight: Some(FontWeight::Bold),
                font_style: None,
                size_pixels: None,
                color: None,
            });
        let diagnostic = emit_all(&plan, &bindings).unwrap_err().diagnostics()[0].clone();
        assert_eq!(diagnostic.kind, CodegenErrorKind::UnsupportedCaptionFeature);
        assert_eq!(diagnostic.code, "CAPTION_RICH_TEXT_UNSUPPORTED");
    }
}

#[test]
fn plain_sidecars_normalize_cr_and_reject_unsafe_or_inexact_cues() {
    let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::Srt);
    first(&mut plan).0.text = "First\rline".to_owned();
    assert!(content(&emit_all(&plan, &bindings).unwrap()).contains("First\nline"));

    for unsafe_text in ["First\n\nline", "First\0line"] {
        let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::Srt);
        first(&mut plan).0.text = unsafe_text.to_owned();
        assert_eq!(
            emit_all(&plan, &bindings).unwrap_err().diagnostics()[0].code,
            "CAPTION_TEXT_UNSAFE"
        );
    }

    let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::WebVtt);
    caption_clip(&mut plan).record_range.start = RationalTime::new(301, 600).unwrap();
    assert_eq!(
        emit_all(&plan, &bindings).unwrap_err().diagnostics()[0].code,
        "PLAN_CAPTION_SIDECAR_INVALID"
    );
}

fn first(plan: &mut ResolvedRenderPlan) -> (&mut veac_plan::ResolvedText, &mut Option<String>) {
    let ResolvedClipSource::Caption {
        content, speaker, ..
    } = &mut caption_clip(plan).source
    else {
        unreachable!()
    };
    (content, speaker)
}

fn caption_clip(plan: &mut ResolvedRenderPlan) -> &mut veac_plan::ResolvedClip {
    plan.sequences[0]
        .tracks
        .iter_mut()
        .find(|track| track.id.as_str() == "trk_caption")
        .unwrap()
        .clips
        .first_mut()
        .unwrap()
}
