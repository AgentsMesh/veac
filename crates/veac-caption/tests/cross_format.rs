use veac_caption::*;

const SRT: &str = "1\n00:00:00,000 --> 00:00:01,250\n<b>你好 VEAC</b>\n第二行\n\n2\n00:00:02,000 --> 00:00:03,000\nDone\n";

#[test]
fn srt_to_vtt_to_ass_preserves_timeline_and_plain_text() {
    let source = import_caption(SRT, CaptionFormat::Srt, &ImportOptions::default()).unwrap();
    let expected: Vec<_> = source
        .document
        .cues
        .iter()
        .map(|cue| (cue.text.plain.clone(), cue.range))
        .collect();

    let vtt = export_caption(
        &CaptionEnvelope::new(source.document),
        CaptionFormat::WebVtt,
    )
    .unwrap();
    let from_vtt = import_caption(
        &vtt.content,
        CaptionFormat::WebVtt,
        &ImportOptions::default(),
    )
    .unwrap();
    assert_eq!(from_vtt.document.cues[0].text.plain, expected[0].0);
    assert_eq!(from_vtt.document.cues[0].range, expected[0].1);

    let ass = export_caption(&CaptionEnvelope::new(from_vtt.document), CaptionFormat::Ass).unwrap();
    let from_ass =
        import_caption(&ass.content, CaptionFormat::Ass, &ImportOptions::default()).unwrap();
    assert_eq!(from_ass.document.cues.len(), 2);
    assert_eq!(from_ass.document.cues[0].text.plain, "你好 VEAC\n第二行");
    assert_eq!(from_ass.document.cues[1].range, expected[1].1);
}

#[test]
fn vtt_speaker_and_settings_report_srt_loss() {
    let vtt = "WEBVTT\n\ncue-a\n00:00:00.000 --> 00:00:01.000 line:80%\n<v Alice>Hello</v>\n";
    let imported = import_caption(vtt, CaptionFormat::WebVtt, &ImportOptions::default()).unwrap();
    let output =
        export_caption(&CaptionEnvelope::new(imported.document), CaptionFormat::Srt).unwrap();
    let fields: Vec<_> = output
        .loss_report
        .losses
        .iter()
        .map(|loss| loss.field.as_str())
        .collect();
    assert!(fields.contains(&"speaker"));
    assert!(fields.contains(&"settings.webvtt.identifier"));
    assert!(fields.contains(&"settings.webvtt.settings"));
    assert!(output.content.contains("Hello"));
}

#[test]
fn canonical_serialization_survives_cross_format_import() {
    let imported = import_caption(SRT, CaptionFormat::Srt, &ImportOptions::default()).unwrap();
    let envelope = CaptionEnvelope::new(imported.document);
    let json = canonical_caption_json(&envelope).unwrap();
    assert_eq!(decode_caption_json(&json).unwrap(), envelope);
    assert_eq!(caption_hash(&envelope).unwrap().len(), 64);
}
