use std::collections::BTreeMap;

use crate::{unit_tests::support::*, *};

#[test]
fn veac_extension_round_trips_sequence_materials_and_multicam_exactly() {
    let project = project();
    let exported = export_sequence(&project, &project.project.entry_sequence_id).unwrap();
    let imported = import_veac_extension(&exported.timeline).unwrap();
    assert_eq!(imported.sequence, project.project.sequences[0]);
    assert_eq!(imported.materials, project.project.materials);
    assert!(imported.loss_report.is_empty());
}

#[test]
fn bound_standard_otio_requires_exact_bindings_and_reports_projection_loss() {
    let project = project();
    let mut timeline = export_sequence(&project, &project.project.entry_sequence_id)
        .unwrap()
        .timeline;
    timeline.metadata.clear();
    let bindings = OtioImportBindings {
        timebase: 600,
        sequence_id: veac_ir::SequenceId::new("seq_imported").unwrap(),
        sequence_name: "Imported".to_owned(),
        settings: settings(),
        tracks: vec![OtioTrackBinding {
            track_id: veac_ir::TrackId::new("trk_imported").unwrap(),
            order: 2,
        }],
        clip_ids: BTreeMap::from([(
            "/tracks/0/children/1".to_owned(),
            veac_ir::ItemId::new("itm_imported").unwrap(),
        )]),
        media: vec![OtioMediaBinding {
            target_url: "media/video.mp4".to_owned(),
            material: material(
                "med_imported",
                "media/video.mp4",
                veac_ir::MaterialKind::Video,
            ),
        }],
    };
    let imported = import_bound_timeline(&timeline, &bindings).unwrap();
    let veac_ir::SequenceAuthorship::Otio { document_sha256 } =
        imported.sequence.authorship.as_ref().unwrap()
    else {
        panic!("bound OTIO import must publish typed digest authorship")
    };
    assert_eq!(document_sha256.as_str().len(), 64);
    assert!(imported.sequence.tracks[0].clips[0].authorship.is_none());
    assert!(imported.relations.is_empty());
    assert_eq!(
        imported.sequence.tracks[0].clips[0]
            .record_range
            .start
            .value,
        300
    );
    assert_eq!(
        imported.sequence.tracks[0].clips[0]
            .source_mapping
            .as_ref()
            .unwrap()
            .time_map,
        veac_ir::SourceTimeMap::Linear {
            source_start: time(1200),
            rate: veac_ir::Rational::new(1, 1).unwrap(),
            repeat: 1,
            direction: veac_ir::PlaybackDirection::Forward,
        }
    );
    assert!(!imported.loss_report.is_empty());

    let mut stale = bindings.clone();
    stale.clip_ids.insert(
        "/stale".to_owned(),
        veac_ir::ItemId::new("itm_stale").unwrap(),
    );
    assert!(import_bound_timeline(&timeline, &stale).is_err());
}
