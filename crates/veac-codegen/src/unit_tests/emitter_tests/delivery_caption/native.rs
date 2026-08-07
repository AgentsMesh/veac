use veac_codegen::emitter::emit_all;
use veac_plan::canonical::*;
use veac_plan::ResolvedClipSource;

use super::{caption_plan, content};

#[test]
fn native_srt_and_webvtt_identity_and_settings_are_preserved() {
    let (mut srt, bindings, _) = caption_plan(CaptionSidecarFormat::Srt);
    cue(&mut srt).native = Some(CaptionNativeCue::Srt { index: 41 });
    assert!(content(&emit_all(&srt, &bindings).unwrap()).starts_with("41\n"));

    let (mut webvtt, bindings, _) = caption_plan(CaptionSidecarFormat::WebVtt);
    cue(&mut webvtt).native = Some(CaptionNativeCue::WebVtt {
        identifier: Some(CaptionNativeId("cue-a".to_owned())),
        settings: Some(WebVttCueSettings {
            line: Some("20%".to_owned()),
            position: Some("30%".to_owned()),
            size: Some("40%".to_owned()),
            align: Some(WebVttTextAlign::Start),
            vertical: Some(WebVttVertical::Rl),
            region: Some(WebVttRegionId("main".to_owned())),
        }),
    });
    let output = content(&emit_all(&webvtt, &bindings).unwrap());
    assert!(output.contains("cue-a\n"));
    assert!(
        output.contains(" line:20% position:30% size:40% align:start vertical:rl region:main\n")
    );
}

#[test]
fn native_ass_event_controls_are_preserved() {
    let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::Ass);
    super::ass::support::normalize(&mut plan);
    cue(&mut plan).native = Some(CaptionNativeCue::Ass {
        settings: AssCueSettings {
            comment: true,
            layer: Some(7),
            margin_left: Some(11),
            margin_right: Some(12),
            margin_vertical: Some(13),
            effect: Some("Banner".to_owned()),
        },
    });
    let output = content(&emit_all(&plan, &bindings).unwrap());
    assert!(output.contains("Comment: 7,"));
    assert!(output.contains(",11,12,13,Banner,"));
}

#[test]
fn unsupported_native_semantics_and_format_conversion_fail_closed() {
    for (mutate, code) in [
        (
            add_span as fn(&mut CaptionCueSemantics),
            "CAPTION_NATIVE_SPANS_UNSUPPORTED",
        ),
        (add_word, "CAPTION_WORD_TIMING_UNSUPPORTED"),
        (add_style, "CAPTION_NATIVE_STYLE_UNSUPPORTED"),
        (add_webvtt, "CAPTION_NATIVE_FORMAT_MISMATCH"),
    ] {
        let (mut plan, bindings, _) = caption_plan(CaptionSidecarFormat::Srt);
        mutate(cue(&mut plan));
        let error = emit_all(&plan, &bindings).unwrap_err();
        assert!(error.diagnostics().iter().any(|value| value.code == code));
    }
}

fn cue(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut CaptionCueSemantics {
    let track = plan.sequences[0]
        .tracks
        .iter_mut()
        .find(|track| track.id.as_str() == "trk_caption")
        .expect("caption track");
    let source = &mut track.clips[0].source;
    let ResolvedClipSource::Caption { cue, .. } = source else {
        panic!("caption fixture")
    };
    cue
}

fn add_span(value: &mut CaptionCueSemantics) {
    value.spans.push(CaptionMarkupSpan {
        range: CaptionTextRange { start: 0, end: 1 },
        style: CaptionInlineStyle {
            bold: true,
            ..Default::default()
        },
    });
}

fn add_word(value: &mut CaptionCueSemantics) {
    value.words.push(CaptionWordTiming {
        text: "Second".to_owned(),
        range: TimeRange::new(super::time(600), super::time(100)).unwrap(),
        confidence: Some(1.0),
    });
}

fn add_style(value: &mut CaptionCueSemantics) {
    value.style = Some(CaptionStyleReference("Default".to_owned()));
}

fn add_webvtt(value: &mut CaptionCueSemantics) {
    value.native = Some(CaptionNativeCue::WebVtt {
        identifier: None,
        settings: None,
    });
}
