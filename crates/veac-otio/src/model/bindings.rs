use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{ItemId, Material, SequenceId, SequenceSettings, TrackId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OtioImportBindings {
    pub timebase: u32,
    pub sequence_id: SequenceId,
    pub sequence_name: String,
    pub settings: SequenceSettings,
    /// One binding per OTIO track, in source order.
    pub tracks: Vec<OtioTrackBinding>,
    /// Keys are JSON pointers such as `/tracks/0/children/2`.
    pub clip_ids: BTreeMap<String, ItemId>,
    /// Sorted by `target_url`; each URL must resolve to one canonical material.
    pub media: Vec<OtioMediaBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OtioTrackBinding {
    pub track_id: TrackId,
    pub order: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OtioMediaBinding {
    pub target_url: String,
    pub material: Material,
}

impl OtioImportBindings {
    pub(crate) fn validate(&self) -> Result<(), crate::OtioError> {
        if self.timebase == 0
            || self.sequence_name.trim().is_empty()
            || self.settings.width == 0
            || self.settings.height == 0
            || self.settings.sample_rate == 0
            || !self.settings.frame_rate.is_positive()
        {
            return Err(crate::OtioError::contract(
                "import sequence settings and name must be valid",
            ));
        }
        if !self
            .media
            .windows(2)
            .all(|pair| pair[0].target_url < pair[1].target_url)
            || self.media.iter().any(|item| item.target_url.is_empty())
        {
            return Err(crate::OtioError::contract(
                "media bindings must be non-empty, unique, and sorted by target URL",
            ));
        }
        let track_ids = self
            .tracks
            .iter()
            .map(|item| item.track_id.as_str())
            .collect::<BTreeSet<_>>();
        let orders = self
            .tracks
            .iter()
            .map(|item| item.order)
            .collect::<BTreeSet<_>>();
        if track_ids.len() != self.tracks.len() || orders.len() != self.tracks.len() {
            return Err(crate::OtioError::contract(
                "track bindings must use unique IDs and orders",
            ));
        }
        let material_ids = self
            .media
            .iter()
            .map(|item| item.material.id.as_str())
            .collect::<BTreeSet<_>>();
        if material_ids.len() != self.media.len() {
            return Err(crate::OtioError::contract(
                "media bindings must use unique material IDs",
            ));
        }
        let clip_ids = self
            .clip_ids
            .values()
            .map(|item| item.as_str())
            .collect::<BTreeSet<_>>();
        if clip_ids.len() != self.clip_ids.len()
            || self
                .clip_ids
                .keys()
                .any(|pointer| !valid_clip_pointer(pointer))
        {
            return Err(crate::OtioError::contract(
                "clip bindings must use unique IDs and canonical child pointers",
            ));
        }
        Ok(())
    }
}

fn valid_clip_pointer(value: &str) -> bool {
    let parts = value.split('/').collect::<Vec<_>>();
    parts.len() == 5
        && parts[0].is_empty()
        && parts[1] == "tracks"
        && canonical_index(parts[2])
        && parts[3] == "children"
        && canonical_index(parts[4])
}

fn canonical_index(value: &str) -> bool {
    value
        .parse::<usize>()
        .is_ok_and(|index| index.to_string() == value)
}
