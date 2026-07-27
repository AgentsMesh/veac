use std::collections::BTreeMap;

use veac_ir::{Project, RationalTime, Track, TrackKind};

use crate::{OtioError, OtioItem, OtioLossReport, OtioTrack, OTIO_TRACK_SCHEMA};

pub(super) fn export(
    project: &Project,
    track: &Track,
    index: usize,
    losses: &mut OtioLossReport,
) -> Result<OtioTrack, OtioError> {
    let pointer = format!("/tracks/{index}");
    let mut clips = track.clips.iter().collect::<Vec<_>>();
    clips.sort_by(|left, right| {
        left.record_range
            .start
            .partial_cmp(&right.record_range.start)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut cursor = RationalTime::zero(project.timebase).map_err(OtioError::time)?;
    let mut children = Vec::new();
    for clip in clips {
        if clip.record_range.start > cursor {
            children.push(gap(cursor, clip.record_range.start)?);
        } else if clip.record_range.start < cursor {
            losses.push(
                &pointer,
                "clips.record_range",
                "OTIO tracks cannot project overlapping VEAC clips sequentially",
                true,
            );
        }
        let child_index = children.len();
        children.push(super::clip::export(
            project,
            clip,
            &format!("{pointer}/children/{child_index}"),
            losses,
        )?);
        let end = clip.record_range.end().map_err(OtioError::time)?;
        if end > cursor {
            cursor = end;
        }
    }
    if track.kind == TrackKind::Visual || track.kind == TrackKind::Caption {
        losses.push(
            &pointer,
            "kind",
            "OTIO projects visual and caption tracks as Video tracks",
            true,
        );
    }
    report_track_features(track, &pointer, losses);
    Ok(OtioTrack {
        schema: OTIO_TRACK_SCHEMA.to_owned(),
        name: track.id.to_string(),
        kind: if track.kind == TrackKind::Audio {
            "Audio".to_owned()
        } else {
            "Video".to_owned()
        },
        children,
        source_range: None,
        metadata: BTreeMap::from([(
            "veac.track_id".to_owned(),
            serde_json::Value::String(track.id.to_string()),
        )]),
        effects: vec![],
        markers: vec![],
        enabled: track.state.enabled,
        extra: BTreeMap::new(),
    })
}

fn report_track_features(track: &Track, pointer: &str, losses: &mut OtioLossReport) {
    for (present, field, reason) in [
        (
            track.placement_mode != veac_ir::PlacementMode::Free,
            "placement_mode",
            "OTIO has no VEAC magnetic placement contract",
        ),
        (
            track.state.muted || track.state.solo || track.state.locked,
            "state",
            "OTIO only standardizes the enabled track state",
        ),
        (
            track.routing != veac_ir::TrackRouting::Default,
            "routing",
            "OTIO has no VEAC audio-bus routing contract",
        ),
    ] {
        if present {
            losses.push(pointer, field, reason, true);
        }
    }
}

fn gap(start: RationalTime, end: RationalTime) -> Result<OtioItem, OtioError> {
    debug_assert_eq!(start.timescale, end.timescale);
    let value = end.value - start.value;
    let duration = RationalTime::new(value, start.timescale).map_err(OtioError::time)?;
    Ok(OtioItem::Gap {
        name: "gap".to_owned(),
        source_range: crate::time::export_range(
            veac_ir::TimeRange::new(
                RationalTime::zero(start.timescale).map_err(OtioError::time)?,
                duration,
            )
            .map_err(OtioError::time)?,
        )?,
        metadata: BTreeMap::new(),
        effects: vec![],
        markers: vec![],
        enabled: true,
        extra: BTreeMap::new(),
    })
}
