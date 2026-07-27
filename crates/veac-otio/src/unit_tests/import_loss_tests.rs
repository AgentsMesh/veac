use crate::{unit_tests::import_error_tests::standard, *};

#[test]
fn bound_import_reports_every_ignored_standard_container_field() {
    let (mut timeline, bindings) = standard();
    let range = match &timeline.tracks.children[0].children[0] {
        OtioItem::Gap { source_range, .. } => source_range.clone(),
        _ => unreachable!(),
    };
    timeline.global_start_time = Some(range.start_time.clone());
    timeline
        .metadata
        .insert("owner".to_owned(), "editor".into());
    timeline.extra.insert("custom".to_owned(), true.into());
    timeline.tracks.source_range = Some(range.clone());
    timeline
        .tracks
        .metadata
        .insert("stack".to_owned(), 1.into());
    timeline.tracks.effects.push("stack-effect".into());
    timeline.tracks.markers.push("stack-marker".into());
    timeline.tracks.enabled = false;
    let track = &mut timeline.tracks.children[0];
    track.source_range = Some(range);
    track.metadata.insert("track".to_owned(), 1.into());
    track.effects.push("track-effect".into());
    track.markers.push("track-marker".into());
    track.extra.insert("track-extra".to_owned(), true.into());
    let OtioItem::Gap {
        metadata,
        effects,
        markers,
        enabled,
        extra,
        ..
    } = &mut track.children[0]
    else {
        unreachable!()
    };
    metadata.insert("gap".to_owned(), 1.into());
    effects.push("gap-effect".into());
    markers.push("gap-marker".into());
    *enabled = false;
    extra.insert("gap-extra".to_owned(), true.into());
    let OtioItem::Clip {
        effects,
        markers,
        extra,
        media_reference,
        ..
    } = &mut track.children[1]
    else {
        unreachable!()
    };
    effects.push("clip-effect".into());
    markers.push("clip-marker".into());
    extra.insert("clip-extra".to_owned(), true.into());
    let OtioMediaReference::External {
        available_range,
        metadata,
        extra,
        ..
    } = media_reference
    else {
        unreachable!()
    };
    *available_range = Some(
        crate::time::export_range(
            veac_ir::TimeRange::new(
                veac_ir::RationalTime::zero(600).unwrap(),
                veac_ir::RationalTime::new(600, 600).unwrap(),
            )
            .unwrap(),
        )
        .unwrap(),
    );
    metadata.insert("reference".to_owned(), 1.into());
    extra.insert("reference-extra".to_owned(), true.into());

    let imported = import_bound_timeline(&timeline, &bindings).unwrap();
    let fields = imported
        .loss_report
        .losses
        .iter()
        .map(|loss| loss.field.as_str())
        .collect::<Vec<_>>();
    for field in [
        "global_start_time",
        "source_range",
        "effects",
        "markers",
        "enabled",
        "media_reference.available_range",
        "media_reference.metadata",
        "media_reference.extra",
    ] {
        assert!(fields.contains(&field), "missing {field}: {fields:?}");
    }
}
