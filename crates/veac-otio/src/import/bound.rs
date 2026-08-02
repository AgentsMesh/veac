mod clip;
mod report;

use std::collections::{BTreeMap, BTreeSet};

use veac_ir::*;

use crate::{
    OtioError, OtioImportBindings, OtioImportResult, OtioItem, OtioLossReport, OtioTimeline,
};

pub(super) struct Context<'a> {
    bindings: &'a OtioImportBindings,
    media: BTreeMap<&'a str, &'a Material>,
    used_clips: BTreeSet<String>,
    used_media: BTreeSet<String>,
    losses: OtioLossReport,
}

pub(super) fn import(
    timeline: &OtioTimeline,
    bindings: &OtioImportBindings,
) -> Result<OtioImportResult, OtioError> {
    if timeline.tracks.children.len() != bindings.tracks.len() {
        return Err(OtioError::contract(
            "track bindings must match the OTIO track count",
        ));
    }
    let mut context = Context {
        bindings,
        media: bindings
            .media
            .iter()
            .map(|item| (item.target_url.as_str(), &item.material))
            .collect(),
        used_clips: BTreeSet::new(),
        used_media: BTreeSet::new(),
        losses: OtioLossReport::default(),
    };
    let mut tracks = Vec::with_capacity(bindings.tracks.len());
    for (index, (track, binding)) in timeline
        .tracks
        .children
        .iter()
        .zip(&bindings.tracks)
        .enumerate()
    {
        tracks.push(import_track(track, binding, index, &mut context)?);
    }
    if context.used_clips.len() != bindings.clip_ids.len()
        || context.used_media.len() != bindings.media.len()
    {
        return Err(OtioError::contract(
            "clip and media bindings must be exact; stale entries are rejected",
        ));
    }
    super::report::extras(&mut context.losses, "", &timeline.extra);
    super::report::extras(&mut context.losses, "/tracks", &timeline.tracks.extra);
    super::report::values(
        &mut context.losses,
        "/tracks",
        "effects",
        &timeline.tracks.effects,
    );
    super::report::values(
        &mut context.losses,
        "/tracks",
        "markers",
        &timeline.tracks.markers,
    );
    report::timeline(timeline, &mut context.losses);
    report::annotations(timeline, &mut context.losses);
    let metadata = BTreeMap::from([("otio.timeline".to_owned(), serde_json::to_value(timeline)?)]);
    Ok(OtioImportResult {
        sequence: Sequence {
            id: bindings.sequence_id.clone(),
            name: bindings.sequence_name.clone(),
            settings: bindings.settings.clone(),
            tracks,
            applies: vec![],
            metadata,
        },
        relations: vec![],
        materials: bindings
            .media
            .iter()
            .filter(|item| context.used_media.contains(item.target_url.as_str()))
            .map(|item| item.material.clone())
            .collect(),
        multicam_groups: vec![],
        annotations: vec![],
        loss_report: context.losses,
    })
}

fn import_track(
    track: &crate::OtioTrack,
    binding: &crate::OtioTrackBinding,
    index: usize,
    context: &mut Context<'_>,
) -> Result<Track, OtioError> {
    let pointer = format!("/tracks/{index}");
    let kind = track_kind(&track.kind, &pointer, &mut context.losses);
    let timebase = context.bindings.timebase;
    let mut cursor = RationalTime::zero(timebase).map_err(OtioError::time)?;
    let mut clips = Vec::new();
    for (child_index, item) in track.children.iter().enumerate() {
        let child_pointer = format!("{pointer}/children/{child_index}");
        match item {
            OtioItem::Gap { source_range, .. } => {
                report::gap(item, &child_pointer, &mut context.losses);
                let range = crate::time::import_range(source_range.clone(), timebase)?;
                cursor = cursor
                    .checked_add(range.duration)
                    .map_err(OtioError::time)?;
            }
            OtioItem::Clip { .. } => {
                let clip = clip::import(
                    item,
                    kind,
                    binding.order,
                    cursor,
                    timebase,
                    &child_pointer,
                    context,
                )?;
                cursor = cursor
                    .checked_add(clip.record_range.duration)
                    .map_err(OtioError::time)?;
                clips.push(clip);
            }
            OtioItem::Transition { .. } | OtioItem::Unknown => context.losses.push(
                &child_pointer,
                "OTIO_SCHEMA",
                "transition or unknown item is omitted without an explicit mapping",
                false,
            ),
        }
    }
    super::report::extras(&mut context.losses, &pointer, &track.extra);
    super::report::values(&mut context.losses, &pointer, "effects", &track.effects);
    super::report::values(&mut context.losses, &pointer, "markers", &track.markers);
    report::track(track, &pointer, &mut context.losses);
    Ok(Track {
        id: binding.track_id.clone(),
        kind,
        order: binding.order,
        placement_mode: PlacementMode::Free,
        state: TrackState {
            enabled: track.enabled,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips,
    })
}

fn track_kind(value: &str, pointer: &str, losses: &mut OtioLossReport) -> TrackKind {
    match value {
        "Audio" => TrackKind::Audio,
        "Video" => TrackKind::Video,
        _ => {
            losses.push(
                pointer,
                "kind",
                "unknown OTIO track kind projected as Video",
                false,
            );
            TrackKind::Video
        }
    }
}
