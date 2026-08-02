use std::collections::BTreeMap;

use crate::{unit_tests::support::*, *};

pub(super) fn standard() -> (OtioTimeline, OtioImportBindings) {
    let project = project();
    let mut timeline = export_sequence(&project, &project.project.entry_sequence_id)
        .unwrap()
        .timeline;
    timeline.metadata.clear();
    let bindings = OtioImportBindings {
        timebase: 600,
        sequence_id: veac_ir::SequenceId::new("seq_bound").unwrap(),
        sequence_name: "Bound".to_owned(),
        settings: settings(),
        tracks: vec![OtioTrackBinding {
            track_id: veac_ir::TrackId::new("trk_bound").unwrap(),
            order: 0,
        }],
        clip_ids: BTreeMap::from([(
            "/tracks/0/children/1".to_owned(),
            veac_ir::ItemId::new("itm_bound").unwrap(),
        )]),
        media: vec![OtioMediaBinding {
            target_url: "media/video.mp4".to_owned(),
            material: material("med_bound", "bound.mp4", veac_ir::MaterialKind::Video),
        }],
    };
    (timeline, bindings)
}

#[test]
fn bound_import_rejects_missing_stale_and_incompatible_bindings() {
    let (timeline, bindings) = standard();
    let mut value = bindings.clone();
    value.tracks.clear();
    assert!(import_bound_timeline(&timeline, &value).is_err());

    let mut value = bindings.clone();
    value.clip_ids.clear();
    assert!(import_bound_timeline(&timeline, &value).is_err());
    let mut value = bindings.clone();
    value.media.clear();
    assert!(import_bound_timeline(&timeline, &value).is_err());
    let mut value = bindings;
    value.media[0].material.kind = veac_ir::MaterialKind::Audio;
    assert!(import_bound_timeline(&timeline, &value).is_err());
}

#[test]
fn bound_import_maps_audio_and_reports_transition_and_unknown_track_kind() {
    let (mut timeline, mut bindings) = standard();
    timeline.tracks.children[0].kind = "Audio".to_owned();
    bindings.tracks[0].track_id = veac_ir::TrackId::new("trk_audio").unwrap();
    bindings.media[0].material.kind = veac_ir::MaterialKind::Audio;
    let imported = import_bound_timeline(&timeline, &bindings).unwrap();
    assert_eq!(imported.sequence.tracks[0].kind, veac_ir::TrackKind::Audio);
    assert!(imported.sequence.tracks[0].clips[0].audio.is_some());

    timeline.tracks.children[0].kind = "Data".to_owned();
    timeline.tracks.children[0]
        .children
        .push(OtioItem::Transition {
            name: "dissolve".to_owned(),
            in_offset: time_value(3.0),
            out_offset: time_value(3.0),
            metadata: BTreeMap::new(),
            extra: BTreeMap::new(),
        });
    bindings.media[0].material.kind = veac_ir::MaterialKind::Video;
    let imported = import_bound_timeline(&timeline, &bindings).unwrap();
    assert!(imported
        .loss_report
        .losses
        .iter()
        .any(|loss| loss.field == "OTIO_SCHEMA"));
}

#[test]
fn malformed_item_reference_and_time_schemas_fail_closed() {
    let (mut timeline, bindings) = standard();
    let OtioItem::Clip {
        media_reference, ..
    } = &mut timeline.tracks.children[0].children[1]
    else {
        unreachable!()
    };
    *media_reference = OtioMediaReference::Missing {
        metadata: BTreeMap::new(),
        extra: BTreeMap::new(),
    };
    assert!(import_bound_timeline(&timeline, &bindings).is_err());

    let json = canonical_otio_json(&timeline).unwrap();
    let unknown = json.replacen("\"Gap.1\"", "\"Mystery.1\"", 1);
    assert!(decode_otio_json(&unknown).is_err());
    timeline.global_start_time = Some(OtioRationalTime {
        schema: OTIO_RATIONAL_TIME_SCHEMA.to_owned(),
        value: 0.0,
        rate: -1.0,
    });
    assert!(canonical_otio_json(&timeline).is_err());
}

fn time_value(value: f64) -> OtioRationalTime {
    OtioRationalTime {
        schema: OTIO_RATIONAL_TIME_SCHEMA.to_owned(),
        value,
        rate: 30.0,
    }
}
