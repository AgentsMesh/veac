use crate::{unit_tests::support::*, *};

#[test]
fn export_reports_nonstandard_sequence_track_and_clip_semantics() {
    let mut project = project();
    let sequence = &mut project.project.sequences[0];
    sequence.authorship = Some(veac_ir::SequenceAuthorship::Otio {
        document_sha256: veac_ir::Sha256Digest::new("a".repeat(64)),
    });
    let track = &mut sequence.tracks[0];
    track.state.muted = true;
    track.routing = veac_ir::TrackRouting::AudioBus {
        bus_id: veac_ir::BusId::new("bus_mix").unwrap(),
    };
    track.clips[0]
        .source_mapping
        .as_mut()
        .unwrap()
        .frame_synthesis = veac_ir::FrameSynthesisPolicy::Blend;
    track.kind = veac_ir::TrackKind::Visual;
    track.order = 10;
    let mut audio = track.clone();
    audio.id = veac_ir::TrackId::new("trk_audio").unwrap();
    audio.kind = veac_ir::TrackKind::Audio;
    audio.order = 0;
    audio.clips.clear();
    sequence.tracks.push(audio);
    let result = export_sequence(&project, &project.project.entry_sequence_id).unwrap();
    let fields = result
        .loss_report
        .losses
        .iter()
        .map(|loss| loss.field.as_str())
        .collect::<Vec<_>>();
    for field in [
        "settings",
        "authorship",
        "kind",
        "state",
        "routing",
        "visual",
        "source_mapping.frame_synthesis",
    ] {
        assert!(fields.contains(&field), "missing {field}: {fields:?}");
    }
    assert_eq!(result.timeline.tracks.children[0].name, "trk_audio");
    assert_eq!(result.timeline.tracks.children[0].kind, "Audio");
    assert_eq!(result.timeline.tracks.children[1].kind, "Video");
    assert!(result
        .loss_report
        .losses
        .iter()
        .all(|loss| loss.preserved_in_extension));
}

#[test]
fn export_uses_actual_gap_child_pointer_and_reports_overlap_and_nonmedia() {
    let mut project = project();
    let track = &mut project.project.sequences[0].tracks[0];
    track.clips[0].source = veac_ir::ClipSource::Generated {
        generator: veac_ir::Generator::Solid {
            color: veac_ir::Color {
                red: 255,
                green: 0,
                blue: 0,
                alpha: 255,
            },
        },
    };
    track.clips[0].source_mapping = None;
    let mut overlap = clip();
    overlap.id = veac_ir::ItemId::new("itm_overlap").unwrap();
    overlap.record_range = veac_ir::TimeRange::new(time(600), time(600)).unwrap();
    track.clips.push(overlap);
    let result = export_sequence(&project, &project.project.entry_sequence_id).unwrap();
    assert!(matches!(
        result.timeline.tracks.children[0].children[0],
        OtioItem::Gap { .. }
    ));
    assert!(matches!(
        result.timeline.tracks.children[0].children[1],
        OtioItem::Clip {
            media_reference: OtioMediaReference::Missing { .. },
            ..
        }
    ));
    assert!(result
        .loss_report
        .losses
        .iter()
        .any(|loss| loss.pointer == "/tracks/0/children/1" && loss.field == "source"));
    assert!(result
        .loss_report
        .losses
        .iter()
        .any(|loss| loss.field == "clips.record_range"));
}

#[test]
fn extension_rejects_standard_projection_edits_and_bad_payloads() {
    let project = project();
    let mut timeline = export_sequence(&project, &project.project.entry_sequence_id)
        .unwrap()
        .timeline;
    timeline.name.push_str(" edited");
    let error = import_veac_extension(&timeline).unwrap_err();
    assert!(error.to_string().contains("stale"));

    let mut timeline = export_sequence(&project, &project.project.entry_sequence_id)
        .unwrap()
        .timeline;
    timeline.metadata.clear();
    assert!(import_veac_extension(&timeline)
        .unwrap_err()
        .to_string()
        .contains("no VEAC extension"));

    let mut timeline = export_sequence(&project, &project.project.entry_sequence_id)
        .unwrap()
        .timeline;
    timeline.metadata.get_mut("veac").unwrap()["version"] = 4.into();
    assert!(import_veac_extension(&timeline).is_err());
    let extension = timeline.metadata.get_mut("veac").unwrap();
    extension["version"] = 3.into();
    let materials = extension["materials"].as_array_mut().unwrap();
    materials.push(materials[0].clone());
    assert!(import_veac_extension(&timeline).is_err());
}

#[test]
fn source_rates_reverse_and_curves_are_reported_without_approximation() {
    let mut linear = project();
    linear.project.sequences[0].tracks[0].clips[0]
        .source_mapping
        .as_mut()
        .unwrap()
        .time_map = veac_ir::SourceTimeMap::Linear {
        source_start: time(1200),
        rate: veac_ir::Rational::new(2, 1).unwrap(),
        repeat: 2,
        direction: veac_ir::PlaybackDirection::Reverse,
    };
    let result = export_sequence(&linear, &linear.project.entry_sequence_id).unwrap();
    assert!(result
        .loss_report
        .losses
        .iter()
        .any(|loss| loss.field == "source_mapping"));

    let mut curve = project();
    curve.project.sequences[0].tracks[0].clips[0]
        .source_mapping
        .as_mut()
        .unwrap()
        .time_map = veac_ir::SourceTimeMap::Curve {
        segments: vec![veac_ir::SourceTimeSegment {
            record_duration: time(600),
            source_start: time(1200),
            source_end: time(1800),
            interpolation: veac_ir::SourceTimeInterpolation::Linear,
        }],
    };
    let result = export_sequence(&curve, &curve.project.entry_sequence_id).unwrap();
    assert!(result
        .loss_report
        .losses
        .iter()
        .any(|loss| loss.reason.contains("source-time curve")));
}

#[test]
fn loss_report_helpers_are_deterministic() {
    let mut report = OtioLossReport::default();
    report.push("/x", "field", "reason", false);
    assert_eq!(report.len(), 1);
    assert!(!report.is_empty());
    let json = canonical_otio_loss_json(&report).unwrap();
    assert_eq!(
        serde_json::from_str::<OtioLossReport>(&json).unwrap(),
        report
    );
}
