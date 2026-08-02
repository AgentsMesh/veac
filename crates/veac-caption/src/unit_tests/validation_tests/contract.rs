use std::collections::BTreeMap;

use crate::{test_support::*, *};

fn codes(value: &CaptionEnvelope) -> Vec<String> {
    validate(value)
        .unwrap_err()
        .0
        .into_iter()
        .map(|issue| issue.code)
        .collect()
}

#[test]
fn accepts_complete_document() {
    assert!(validate(&envelope()).is_ok());
}

#[test]
fn validates_envelope_document_and_styles() {
    let mut value = envelope();
    value.schema = "wrong".to_owned();
    value.schema_version = 99;
    value.document.timescale = 0;
    value.document.language = Some(" ".to_owned());
    value.document.settings.insert(String::new(), String::new());
    let mut duplicate = value.document.styles[0].clone();
    duplicate.font_family.clear();
    duplicate.font_size_pixels = 0;
    duplicate.alignment = 10;
    duplicate.border_style = 2;
    duplicate.scale_x_percent = 0.0;
    duplicate.scale_y_percent = f64::NAN;
    duplicate.outline_pixels = -1.0;
    duplicate.shadow_pixels = -1.0;
    value.document.styles.push(duplicate);
    let found = codes(&value);
    for expected in [
        "SCHEMA",
        "VERSION",
        "TIMESCALE",
        "EMPTY",
        "SETTING",
        "DUPLICATE_ID",
        "STYLE_RANGE",
        "STYLE_NUMBER",
    ] {
        assert!(
            found.iter().any(|code| code == expected),
            "missing {expected}: {found:?}"
        );
    }
}

#[test]
fn validates_cue_identity_text_references_spans_and_words() {
    let mut value = envelope();
    let cue = &mut value.document.cues[0];
    cue.id = serde_json::from_str("\"bad\"").unwrap();
    cue.text.plain = " ".to_owned();
    cue.speaker = Some(String::new());
    cue.style = Some("Missing".to_owned());
    cue.settings = BTreeMap::from([(String::new(), String::new())]);
    cue.text.spans = vec![CaptionSpan {
        range: TextRange { start: 2, end: 1 },
        style: InlineStyle::default(),
    }];
    cue.words = vec![
        CaptionWord {
            text: String::new(),
            range: range(900, 200),
            confidence: Some(f64::NAN),
        },
        CaptionWord {
            text: "back".to_owned(),
            range: range(800, 100),
            confidence: Some(1.5),
        },
    ];
    let found = codes(&value);
    for expected in [
        "ID",
        "EMPTY",
        "STYLE_REF",
        "SETTING",
        "TEXT_RANGE",
        "EMPTY_STYLE",
        "WORD_CONTAINMENT",
        "WORD_ORDER",
        "CONFIDENCE",
    ] {
        assert!(
            found.iter().any(|code| code == expected),
            "missing {expected}: {found:?}"
        );
    }
}

#[test]
fn validates_range_sorting_overlap_and_overflow() {
    let mut value = envelope();
    value.document.cues[1].range.start = time(500);
    let found = codes(&value);
    assert!(found.contains(&"OVERLAP".to_owned()));

    value.document.cues.swap(0, 1);
    assert!(codes(&value).contains(&"ORDER".to_owned()));

    let mut value = envelope();
    value.document.cues[0].range.start = veac_ir::RationalTime::new(-1, 1000).unwrap();
    value.document.cues[0].range.duration = veac_ir::RationalTime {
        value: 0,
        timescale: 999,
    };
    assert!(codes(&value).contains(&"TIME_RANGE".to_owned()));
    assert!(codes(&value).contains(&"TIMESCALE".to_owned()));

    value.document.cues[0].range.start.value = 9_007_199_254_740_991;
    value.document.cues[0].range.duration = time(1);
    assert!(codes(&value).contains(&"TIME_OVERFLOW".to_owned()));
}

#[test]
fn validates_duplicate_cues_and_span_option_text() {
    let mut value = envelope();
    value.document.cues[1].id = value.document.cues[0].id.clone();
    value.document.cues[0].text.spans = vec![CaptionSpan {
        range: TextRange { start: 0, end: 1 },
        style: InlineStyle {
            bold: true,
            color: Some(" ".to_owned()),
            voice: Some(" ".to_owned()),
            ..InlineStyle::default()
        },
    }];
    let found = codes(&value);
    assert!(found.contains(&"DUPLICATE_ID".to_owned()));
    assert!(found.iter().filter(|code| *code == "EMPTY").count() >= 2);
}
