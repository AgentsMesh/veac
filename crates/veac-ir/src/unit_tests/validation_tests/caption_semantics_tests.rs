use super::*;

#[test]
fn typed_caption_semantics_validate_every_native_family() {
    let mut project = sample_project();
    let value = semantics_mut(&mut project);
    value.style = Some(CaptionStyleReference("caption.primary".into()));
    value.spans = vec![CaptionMarkupSpan {
        range: CaptionTextRange { start: 0, end: 5 },
        style: CaptionInlineStyle {
            bold: true,
            color: Some("#ffffff".into()),
            voice: Some("Alice".into()),
            ..CaptionInlineStyle::default()
        },
    }];
    value.words = vec![CaptionWordTiming {
        text: "hello".into(),
        range: crate::test_support::range(0, 300),
        confidence: Some(0.95),
    }];
    value.native = Some(CaptionNativeCue::WebVtt {
        identifier: Some(CaptionNativeId("cue-1".into())),
        settings: Some(WebVttCueSettings {
            line: Some("10".into()),
            position: Some("20%".into()),
            size: Some("80%".into()),
            align: Some(WebVttTextAlign::Center),
            vertical: Some(WebVttVertical::Rl),
            region: Some(WebVttRegionId("main".into())),
        }),
    });
    validate(&project).unwrap();

    semantics_mut(&mut project).native = Some(CaptionNativeCue::Srt { index: 1 });
    validate(&project).unwrap();
    semantics_mut(&mut project).native = Some(CaptionNativeCue::Ass {
        settings: AssCueSettings {
            effect: Some("scroll up".into()),
            ..AssCueSettings::default()
        },
    });
    validate(&project).unwrap();
}

#[test]
fn caption_style_spans_and_words_fail_closed() {
    let mut style = sample_project();
    semantics_mut(&mut style).style = Some(CaptionStyleReference(" ".into()));
    assert_code(&validation_codes(&style), "CAPTION_STYLE_REFERENCE");

    let mut span = sample_project();
    semantics_mut(&mut span).spans = vec![CaptionMarkupSpan {
        range: CaptionTextRange { start: 5, end: 1 },
        style: CaptionInlineStyle {
            color: Some("bad\ncolor".into()),
            voice: Some("bad\nvoice".into()),
            ..CaptionInlineStyle::default()
        },
    }];
    assert_code(&validation_codes(&span), "CAPTION_SPAN");

    let mut word = sample_project();
    semantics_mut(&mut word).words = vec![CaptionWordTiming {
        text: " ".into(),
        range: crate::test_support::range(299, 2),
        confidence: Some(f64::NAN),
    }];
    assert_code(&validation_codes(&word), "CAPTION_WORD");
}

#[test]
fn every_invalid_native_caption_contract_is_rejected() {
    let invalid = [
        CaptionNativeCue::Srt { index: 0 },
        CaptionNativeCue::WebVtt {
            identifier: Some(CaptionNativeId("bad\nid".into())),
            settings: None,
        },
        CaptionNativeCue::WebVtt {
            identifier: None,
            settings: Some(WebVttCueSettings {
                line: Some("bad line".into()),
                position: Some("101%".into()),
                size: Some("not-percent".into()),
                align: None,
                vertical: None,
                region: Some(WebVttRegionId("bad region".into())),
            }),
        },
        CaptionNativeCue::Ass {
            settings: AssCueSettings {
                effect: Some("bad,effect".into()),
                ..AssCueSettings::default()
            },
        },
    ];
    for native in invalid {
        let mut project = sample_project();
        semantics_mut(&mut project).native = Some(native);
        assert_code(&validation_codes(&project), "CAPTION_NATIVE");
    }
}

fn semantics_mut(project: &mut ProjectEnvelope) -> &mut CaptionCueSemantics {
    let ClipSource::Caption { cue, .. } =
        &mut project.project.sequences[0].tracks[1].clips[0].source
    else {
        unreachable!()
    };
    cue.as_mut()
}
