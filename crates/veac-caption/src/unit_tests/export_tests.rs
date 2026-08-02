use crate::{test_support::*, *};

fn rich_value() -> CaptionEnvelope {
    let mut value = envelope();
    let cue = &mut value.document.cues[0];
    cue.speaker = Some("Stone".to_owned());
    cue.text.spans.push(CaptionSpan {
        range: TextRange { start: 0, end: 2 },
        style: InlineStyle {
            bold: true,
            italic: true,
            underline: true,
            color: Some("&H00FFFFFF".to_owned()),
            voice: Some("Narrator".to_owned()),
        },
    });
    cue.words.push(CaptionWord {
        text: "你好".to_owned(),
        range: range(0, 500),
        confidence: Some(0.99),
    });
    value
}

#[test]
fn exports_srt_deterministically_with_explicit_losses() {
    let value = rich_value();
    let first = export_caption(&value, CaptionFormat::Srt).unwrap();
    let second = export_caption(&value, CaptionFormat::Srt).unwrap();
    assert_eq!(first, second);
    assert!(first.content.contains("<b><i><u><font color="));
    assert!(first.content.contains("你好"));
    let fields: Vec<_> = first
        .loss_report
        .losses
        .iter()
        .map(|loss| loss.field.as_str())
        .collect();
    assert!(fields.contains(&"language"));
    assert!(fields.contains(&"styles"));
    assert!(fields.contains(&"speaker"));
    assert!(fields.contains(&"words"));
    assert!(fields.contains(&"text.spans.voice"));
}

#[test]
fn exports_webvtt_with_settings_and_speaker() {
    let mut value = rich_value();
    value
        .document
        .settings
        .insert("webvtt.header".to_owned(), "WEBVTT VEAC".to_owned());
    value.document.cues[0]
        .settings
        .insert("webvtt.settings".to_owned(), "line:80%".to_owned());
    let output = export_caption(&value, CaptionFormat::WebVtt).unwrap();
    assert!(output.content.starts_with("WEBVTT VEAC"));
    assert!(output.content.contains("line:80%"));
    assert!(output.content.contains("<v Stone>"));
    assert!(output.content.contains("<v Narrator>"));
    assert!(output
        .loss_report
        .losses
        .iter()
        .any(|loss| loss.field == "text.spans.color"));
}

#[test]
fn exports_ass_styles_actor_comment_and_multiline() {
    let mut value = rich_value();
    value.document.cues[0]
        .settings
        .insert("ass.comment".to_owned(), "true".to_owned());
    let output = export_caption(&value, CaptionFormat::Ass).unwrap();
    assert!(output.content.contains("Style: Default,Inter,42"));
    assert!(output
        .content
        .contains("Comment: 0,0:00:00.00,0:00:01.00,Default,Stone"));
    assert!(output.content.contains("\\NVEAC"));
    assert!(output.content.contains("{\\b1\\i1\\u1\\c&H00FFFFFF}"));
    assert!(output
        .loss_report
        .losses
        .iter()
        .any(|loss| loss.field == "text.spans.voice"));
}

#[test]
fn reports_unsupported_settings_per_format() {
    let mut value = envelope();
    value
        .document
        .settings
        .insert("custom".to_owned(), "x".to_owned());
    value.document.cues[0]
        .settings
        .insert("srt.index".to_owned(), "1".to_owned());
    value.document.cues[0]
        .settings
        .insert("ass.layer".to_owned(), "2".to_owned());
    for format in [
        CaptionFormat::Srt,
        CaptionFormat::WebVtt,
        CaptionFormat::Ass,
    ] {
        let report = export_caption(&value, format).unwrap().loss_report;
        assert!(!report.is_empty());
        assert!(report
            .losses
            .iter()
            .any(|loss| loss.field == "settings.custom"));
    }
}

#[test]
fn rejects_inexact_millisecond_and_ass_centisecond_times() {
    let mut value = envelope();
    value.document.timescale = 600;
    value.document.cues[0].range = veac_ir::TimeRange {
        start: veac_ir::RationalTime::new(1, 600).unwrap(),
        duration: veac_ir::RationalTime::new(599, 600).unwrap(),
    };
    value.document.cues[1].range.start = veac_ir::RationalTime::new(900, 600).unwrap();
    value.document.cues[1].range.duration = veac_ir::RationalTime::new(600, 600).unwrap();
    assert!(matches!(
        export_caption(&value, CaptionFormat::Srt),
        Err(CaptionError::Time(_))
    ));

    let mut value = envelope();
    value.document.cues[0].range.duration = range(0, 1005).duration;
    value.document.cues[1].range.start = time(1505);
    assert!(matches!(
        export_caption(&value, CaptionFormat::Ass),
        Err(CaptionError::Time(_))
    ));
}
