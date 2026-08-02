use std::collections::BTreeMap;

use crate::{unit_tests::support::*, *};

fn bindings() -> OtioImportBindings {
    OtioImportBindings {
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
    }
}

#[test]
fn bindings_reject_invalid_settings_duplicate_tracks_and_materials() {
    let mut value = bindings();
    value.timebase = 0;
    assert!(value.validate().is_err());

    let mut value = bindings();
    value.tracks.push(value.tracks[0].clone());
    value.tracks[1].order = 1;
    assert!(value.validate().is_err());
    value.tracks[1].track_id = veac_ir::TrackId::new("trk_other").unwrap();
    value.tracks[1].order = 0;
    assert!(value.validate().is_err());

    let mut value = bindings();
    let mut duplicate = value.media[0].clone();
    duplicate.target_url = "z.mp4".to_owned();
    value.media.push(duplicate);
    assert!(value.validate().is_err());
}

#[test]
fn bindings_reject_unsorted_media_and_non_exact_clip_pointers() {
    let mut value = bindings();
    value.media.push(OtioMediaBinding {
        target_url: "a.mp4".to_owned(),
        material: material("med_a", "a.mp4", veac_ir::MaterialKind::Video),
    });
    assert!(value.validate().is_err());

    let mut value = bindings();
    value.clip_ids.insert(
        "/tracks/0/children/2".to_owned(),
        value.clip_ids.values().next().unwrap().clone(),
    );
    assert!(value.validate().is_err());
    value.clip_ids.clear();
    value.clip_ids.insert(
        "/tracks/00/children/1".to_owned(),
        veac_ir::ItemId::new("itm_other").unwrap(),
    );
    assert!(value.validate().is_err());
}
