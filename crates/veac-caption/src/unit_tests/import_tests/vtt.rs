use crate::*;

const VTT: &str = "WEBVTT demo\n\ncue-alpha\n00:00:00.000 --> 00:00:01.000 line:80% align:center\n<v Alice><b>Hello</b> 世界</v>\n\n00:00:01.500 --> 00:00:02.500\n<c.red>color</c>\n";

#[test]
fn imports_vtt_identifiers_settings_speaker_and_rich_text() {
    let result = import_caption(VTT, CaptionFormat::WebVtt, &ImportOptions::default()).unwrap();
    assert_eq!(result.document.cues.len(), 2);
    let cue = &result.document.cues[0];
    assert_eq!(cue.speaker.as_deref(), Some("Alice"));
    assert_eq!(cue.settings["webvtt.identifier"], "cue-alpha");
    assert_eq!(cue.settings["webvtt.settings"], "line:80% align:center");
    assert!(cue.text.spans.iter().any(|span| span.style.bold));
    assert!(result.document.settings["webvtt.header"].contains("WEBVTT"));
    assert_eq!(result.loss_report.losses[0].field, "text.spans");
}

#[test]
fn validates_vtt_structure_and_timestamps() {
    let malformed = "WEBVTT\n\n00:00:00.000 --> nope\nx\n";
    assert!(matches!(
        import_caption(malformed, CaptionFormat::WebVtt, &ImportOptions::default()),
        Err(CaptionError::Parse { .. })
    ));
    let empty = "WEBVTT\n\nNOTE only\n";
    assert!(matches!(
        import_caption(empty, CaptionFormat::WebVtt, &ImportOptions::default()),
        Err(CaptionError::Parse { .. })
    ));
}

#[test]
fn caption_format_public_surface_excludes_ssa() {
    assert!(serde_json::from_str::<CaptionFormat>("\"ssa\"").is_err());
    assert_eq!(CaptionFormat::Srt.to_string(), "SRT");
    assert_eq!(CaptionFormat::WebVtt.to_string(), "WebVTT");
    assert_eq!(CaptionFormat::Ass.to_string(), "ASS");
}
