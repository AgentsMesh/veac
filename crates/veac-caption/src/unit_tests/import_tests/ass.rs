use crate::*;

const ASS: &str = r#"[Script Info]
Title: Demo
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Inter,42,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 2,0:00:00.00,0:00:01.00,Default,Alice,12,0,0,Scroll,{\b1}你好\NVEAC{\r}
Comment: 0,0:00:01.50,0:00:02.50,Default,,0,0,0,,note
"#;

#[test]
fn imports_ass_styles_speaker_extras_comments_and_rich_text() {
    let result = import_caption(ASS, CaptionFormat::Ass, &ImportOptions::default()).unwrap();
    assert_eq!(result.document.settings["ass.info.Title"], "Demo");
    assert_eq!(result.document.styles[0].font_family, "Inter");
    let cue = &result.document.cues[0];
    assert_eq!(cue.speaker.as_deref(), Some("Alice"));
    assert_eq!(cue.style.as_deref(), Some("Default"));
    assert_eq!(cue.settings["ass.layer"], "2");
    assert_eq!(cue.settings["ass.margin_left"], "12");
    assert_eq!(cue.settings["ass.effect"], "Scroll");
    assert_eq!(cue.text.plain, "你好\nVEAC");
    assert!(cue.text.spans[0].style.bold);
    assert_eq!(result.document.cues[1].settings["ass.comment"], "true");
    assert!(result.loss_report.is_empty());
}

#[test]
fn reports_unsupported_ass_overrides() {
    let input = ASS.replace("{\\b1}", "{\\pos(1,2)\\b1}");
    let result = import_caption(&input, CaptionFormat::Ass, &ImportOptions::default()).unwrap();
    assert_eq!(result.loss_report.losses[0].field, "text.spans");
}

#[test]
fn rejects_missing_or_malformed_ass_events() {
    assert!(matches!(
        import_caption(
            "[Script Info]\nTitle: empty\n",
            CaptionFormat::Ass,
            &ImportOptions::default()
        ),
        Err(CaptionError::Parse { .. })
    ));
    let malformed = ASS.replace("0:00:01.00", "bad");
    assert!(matches!(
        import_caption(&malformed, CaptionFormat::Ass, &ImportOptions::default()),
        Err(CaptionError::Parse { .. })
    ));
}
