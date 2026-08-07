use std::collections::BTreeMap;

use veac_ir::{
    AssCueSettings, CaptionCueSemantics, CaptionNativeCue, ClipSource, ItemId, TrackId, TrackKind,
};

use crate::{test_support::*, *};

fn item(value: &str) -> ItemId {
    ItemId::new(value).unwrap()
}

fn track_bindings(value: &CaptionEnvelope) -> CaptionTrackBindings {
    CaptionTrackBindings {
        track_id: TrackId::new("trk_imported_captions").unwrap(),
        cue_item_ids: BTreeMap::from([
            (value.document.cues[0].id.clone(), item("itm_caption_one")),
            (value.document.cues[1].id.clone(), item("itm_caption_two")),
        ]),
        text_style: text_style(),
        visual: visual(),
        order: 7,
    }
}

fn document_bindings(value: &CaptionEnvelope) -> CaptionDocumentBindings {
    CaptionDocumentBindings {
        cue_ids: BTreeMap::from([
            (item("itm_caption_one"), value.document.cues[0].id.clone()),
            (item("itm_caption_two"), value.document.cues[1].id.clone()),
        ]),
        language: value.document.language.clone(),
        overlap_policy: value.document.overlap_policy,
        native: value.document.native.clone(),
        styles: value.document.styles.clone(),
    }
}

#[test]
fn converts_to_and_from_canonical_caption_track_losslessly() {
    let mut value = envelope();
    value.document.cues[0].speaker = Some("Alice".to_owned());
    value.document.cues[0].text.spans.push(CaptionSpan {
        range: TextRange { start: 0, end: 2 },
        style: InlineStyle {
            bold: true,
            ..InlineStyle::default()
        },
    });
    value.document.cues[0].words.push(CaptionWord {
        text: "你好".to_owned(),
        range: range(0, 500),
        confidence: Some(0.9),
    });
    let track = to_caption_track(&value, &track_bindings(&value)).unwrap();
    assert_eq!(track.kind, TrackKind::Caption);
    assert_eq!(track.order, 7);
    assert_eq!(track.clips.len(), 2);
    assert!(
        matches!(&track.clips[0].source, ClipSource::Caption { text, .. } if text == "你好\nVEAC")
    );
    assert!(matches!(
        &track.clips[0].source,
        ClipSource::Caption { speaker, .. } if speaker.as_deref() == Some("Alice")
    ));
    let ClipSource::Caption { cue, .. } = &track.clips[0].source else {
        unreachable!()
    };
    assert_eq!(cue.spans.len(), 1);
    assert_eq!(cue.words.len(), 1);
    assert_eq!(
        from_caption_track(&track, &document_bindings(&value)).unwrap(),
        value
    );
}

#[test]
fn requires_complete_unique_stable_id_bindings() {
    let value = envelope();
    let mut missing = track_bindings(&value);
    missing.cue_item_ids.remove(&value.document.cues[0].id);
    assert!(matches!(
        to_caption_track(&value, &missing),
        Err(CaptionError::Ir(_))
    ));

    let mut duplicate = track_bindings(&value);
    duplicate
        .cue_item_ids
        .insert(value.document.cues[1].id.clone(), item("itm_caption_one"));
    assert!(matches!(
        to_caption_track(&value, &duplicate),
        Err(CaptionError::Ir(_))
    ));
}

#[test]
fn rejects_wrong_track_items_and_bindings() {
    let value = envelope();
    let mut track = to_caption_track(&value, &track_bindings(&value)).unwrap();
    track.kind = TrackKind::Video;
    assert!(matches!(
        from_caption_track(&track, &document_bindings(&value)),
        Err(CaptionError::Ir(_))
    ));

    track.kind = TrackKind::Caption;
    let mut missing = document_bindings(&value);
    missing.cue_ids.remove(&track.clips[0].id);
    assert!(matches!(
        from_caption_track(&track, &missing),
        Err(CaptionError::Ir(_))
    ));

    let mut wrong_source = track.clone();
    wrong_source.clips[0].source = ClipSource::Generated {
        generator: veac_ir::Generator::Transparent,
    };
    assert!(matches!(
        from_caption_track(&wrong_source, &document_bindings(&value)),
        Err(CaptionError::Ir(_))
    ));
}

#[test]
fn plain_ir_caption_semantics_use_defaults() {
    let value = envelope();
    let mut track = to_caption_track(&value, &track_bindings(&value)).unwrap();
    track.clips.iter_mut().for_each(|clip| {
        let ClipSource::Caption { cue, .. } = &mut clip.source else {
            unreachable!()
        };
        **cue = CaptionCueSemantics::default();
    });
    let imported = from_caption_track(&track, &document_bindings(&value)).unwrap();
    assert!(imported
        .document
        .cues
        .iter()
        .all(|cue| cue.text.spans.is_empty()));
    assert!(imported
        .document
        .cues
        .iter()
        .all(|cue| cue.words.is_empty()));
}

#[test]
fn ass_native_fields_roundtrip_without_string_codec() {
    let mut value = envelope();
    value.document.cues[0].native = Some(CaptionNativeCue::Ass {
        settings: AssCueSettings {
            comment: true,
            layer: Some(2),
            margin_left: Some(12),
            margin_right: Some(13),
            margin_vertical: Some(14),
            effect: Some("Scroll".to_owned()),
        },
    });
    let track = to_caption_track(&value, &track_bindings(&value)).unwrap();
    let ClipSource::Caption { cue, .. } = &track.clips[0].source else {
        unreachable!()
    };
    assert!(matches!(cue.native, Some(CaptionNativeCue::Ass { .. })));
    assert_eq!(
        from_caption_track(&track, &document_bindings(&value)).unwrap(),
        value
    );
}
