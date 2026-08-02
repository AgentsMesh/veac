use std::collections::{BTreeMap, BTreeSet};

use crate::{Apply, ApplyId, ItemId, RelationEndpoint, Sequence, TrackRouting};

use super::RelationItem;

pub struct RelationSequence<'a> {
    pub(super) sequence: &'a Sequence,
    pub(super) items: BTreeMap<&'a str, RelationItem<'a>>,
    applies: BTreeMap<&'a str, (&'a Apply, usize)>,
    tracks: BTreeSet<&'a str>,
    buses: BTreeSet<&'a str>,
}

impl<'a> RelationSequence<'a> {
    pub(super) fn new(sequence: &'a Sequence) -> Self {
        let items = sequence
            .tracks
            .iter()
            .flat_map(|track| {
                track.clips.iter().enumerate().map(move |(position, clip)| {
                    (
                        clip.id.as_str(),
                        RelationItem {
                            sequence,
                            track,
                            clip,
                            position,
                        },
                    )
                })
            })
            .collect();
        let tracks = sequence
            .tracks
            .iter()
            .map(|track| track.id.as_str())
            .collect();
        let applies = sequence
            .applies
            .iter()
            .enumerate()
            .map(|(position, apply)| (apply.id.as_str(), (apply, position)))
            .collect();
        let buses = sequence
            .tracks
            .iter()
            .filter_map(|track| match &track.routing {
                TrackRouting::AudioBus { bus_id } => Some(bus_id.as_str()),
                TrackRouting::Default => None,
            })
            .collect();
        Self {
            sequence,
            items,
            applies,
            tracks,
            buses,
        }
    }

    pub fn sequence(&self) -> &'a Sequence {
        self.sequence
    }

    pub fn item(&self, id: &ItemId) -> Option<RelationItem<'a>> {
        self.items.get(id.as_str()).copied()
    }

    pub fn track(&self, id: &crate::TrackId) -> Option<&'a crate::Track> {
        self.sequence.tracks.iter().find(|track| track.id == *id)
    }

    pub fn apply(&self, id: &ApplyId) -> Option<(&'a Apply, usize)> {
        self.applies.get(id.as_str()).copied()
    }

    pub fn bus_tracks(&self, id: &crate::BusId) -> Vec<&'a crate::Track> {
        self.sequence
            .tracks
            .iter()
            .filter(|track| {
                matches!(
                    &track.routing,
                    TrackRouting::AudioBus { bus_id } if bus_id == id
                )
            })
            .collect()
    }

    pub fn contains(&self, endpoint: &RelationEndpoint) -> bool {
        match endpoint {
            RelationEndpoint::Item { item_id } => self.items.contains_key(item_id.as_str()),
            RelationEndpoint::Apply { apply_id } => self.applies.contains_key(apply_id.as_str()),
            RelationEndpoint::Track { track_id } => self.tracks.contains(track_id.as_str()),
            RelationEndpoint::Bus { bus_id } => self.buses.contains(bus_id.as_str()),
        }
    }
}
