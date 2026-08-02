use crate::*;

const SRT: &str = "1\n00:00:00,000 --> 00:00:01,000\n你好 <b>VEAC</b>\n第二行\n\n2\n00:00:01,500 --> 00:00:02,500\n<i>done</i>\n";

#[test]
fn imports_unicode_multiline_rich_srt_with_stable_ids() {
    let options = ImportOptions::default();
    let first = import_caption(SRT, CaptionFormat::Srt, &options).unwrap();
    let second = import_caption(SRT, CaptionFormat::Srt, &options).unwrap();
    assert_eq!(first.document, second.document);
    assert!(first.loss_report.is_empty());
    assert_eq!(first.document.cues[0].text.plain, "你好 VEAC\n第二行");
    let span = &first.document.cues[0].text.spans[0];
    assert_eq!(span.range, TextRange { start: 3, end: 7 });
    assert!(span.style.bold);
    assert_eq!(first.document.cues[0].settings["srt.index"], "1");
}

#[test]
fn import_options_are_exact_and_validated() {
    let changed = ImportOptions {
        id_namespace: "other".to_owned(),
        ..ImportOptions::default()
    };
    let first = import_caption(SRT, CaptionFormat::Srt, &ImportOptions::default()).unwrap();
    let other = import_caption(SRT, CaptionFormat::Srt, &changed).unwrap();
    assert_ne!(first.document.cues[0].id, other.document.cues[0].id);

    let inexact = ImportOptions {
        timescale: 24,
        ..ImportOptions::default()
    };
    assert!(matches!(
        import_caption(
            "1\n00:00:00,001 --> 00:00:01,001\nx\n",
            CaptionFormat::Srt,
            &inexact
        ),
        Err(CaptionError::Time(_))
    ));
    let empty_namespace = ImportOptions {
        id_namespace: " ".to_owned(),
        ..ImportOptions::default()
    };
    assert!(matches!(
        import_caption(SRT, CaptionFormat::Srt, &empty_namespace),
        Err(CaptionError::Parse { .. })
    ));
}

#[test]
fn rejects_empty_malformed_and_invalid_ranges() {
    assert!(matches!(
        import_caption(" ", CaptionFormat::Srt, &ImportOptions::default()),
        Err(CaptionError::Parse { .. })
    ));
    assert!(matches!(
        import_caption(
            "not subtitles",
            CaptionFormat::Srt,
            &ImportOptions::default()
        ),
        Err(CaptionError::Parse { .. })
    ));
    assert!(matches!(
        import_caption(
            "1\n00:00:01,000 --> 00:00:01,000\nx\n",
            CaptionFormat::Srt,
            &ImportOptions::default()
        ),
        Err(CaptionError::Time(_))
    ));
}

#[test]
fn overlap_policy_is_explicit() {
    let input = "1\n00:00:00,000 --> 00:00:02,000\na\n\n2\n00:00:01,000 --> 00:00:03,000\nb\n";
    assert!(matches!(
        import_caption(input, CaptionFormat::Srt, &ImportOptions::default()),
        Err(CaptionError::Validation(_))
    ));
    let allow = ImportOptions {
        overlap_policy: OverlapPolicy::Allow,
        ..ImportOptions::default()
    };
    assert!(import_caption(input, CaptionFormat::Srt, &allow).is_ok());
}
