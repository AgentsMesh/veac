use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn source_media_type(
        &mut self,
        source: &ClipSource,
        track_kind: TrackKind,
        path: &str,
        item_id: &str,
    ) {
        let compatible = match source {
            ClipSource::Media { material_id } => self
                .material_ids
                .get(material_id.as_str())
                .is_none_or(|kind| material_on_track(*kind, track_kind)),
            ClipSource::FreezeFrame { material_id, .. } => self
                .material_ids
                .get(material_id.as_str())
                .is_none_or(|kind| {
                    matches!(kind, MaterialKind::Video | MaterialKind::Image)
                        && matches!(track_kind, TrackKind::Video | TrackKind::Visual)
                }),
            ClipSource::Sequence { .. } => matches!(
                track_kind,
                TrackKind::Video | TrackKind::Visual | TrackKind::Audio
            ),
            ClipSource::Multicam { .. } => {
                matches!(track_kind, TrackKind::Video | TrackKind::Visual)
            }
            ClipSource::Text { .. } => track_kind == TrackKind::Visual,
            ClipSource::Caption { .. } => track_kind == TrackKind::Caption,
            ClipSource::Generated { generator } => match generator {
                Generator::Silence => track_kind == TrackKind::Audio,
                Generator::Solid { .. }
                | Generator::Gradient { .. }
                | Generator::Shape { .. }
                | Generator::Transparent => {
                    matches!(track_kind, TrackKind::Video | TrackKind::Visual)
                }
            },
        };
        if !compatible {
            self.value_error("TRACK_MEDIA_TYPE", path, item_id);
        }
    }
}

fn material_on_track(material: MaterialKind, track: TrackKind) -> bool {
    match track {
        TrackKind::Audio => matches!(material, MaterialKind::Audio | MaterialKind::Video),
        TrackKind::Video | TrackKind::Visual => {
            matches!(material, MaterialKind::Video | MaterialKind::Image)
        }
        TrackKind::Caption => false,
    }
}
