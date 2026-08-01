use std::collections::{BTreeMap, BTreeSet};

use veac_ir::{AudioMixSource, BusId, TimeRange, Track, TrackId, TrackRouting};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct AudioDemand {
    master: bool,
    tracks: BTreeSet<TrackId>,
    buses: BTreeSet<BusId>,
}

impl AudioDemand {
    pub(super) fn master() -> Self {
        Self {
            master: true,
            ..Self::default()
        }
    }

    pub(super) fn set_master(&mut self) {
        self.master = true;
    }

    pub(super) fn selects(&self, track: &Track) -> bool {
        self.master
            || self.tracks.contains(&track.id)
            || match &track.routing {
                TrackRouting::AudioBus { bus_id } => self.buses.contains(bus_id),
                TrackRouting::Default => false,
            }
    }

    pub(super) fn add(&mut self, source: &AudioMixSource) {
        match source {
            AudioMixSource::Master => self.master = true,
            AudioMixSource::Track { track_id } => {
                self.tracks.insert(track_id.clone());
            }
            AudioMixSource::Bus { bus_id } => {
                self.buses.insert(bus_id.clone());
            }
        }
    }

    pub(super) fn merge(&mut self, other: &Self) -> bool {
        let before = self.clone();
        self.master |= other.master;
        self.tracks.extend(other.tracks.iter().cloned());
        self.buses.extend(other.buses.iter().cloned());
        *self != before
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct AudioControls {
    all_tracks: Vec<TimeRange>,
    tracks: BTreeMap<TrackId, Vec<TimeRange>>,
}

impl AudioControls {
    pub(super) fn all_tracks(ranges: Vec<TimeRange>) -> Self {
        Self {
            all_tracks: super::super::super::apply_ranges::merge(ranges),
            ..Self::default()
        }
    }

    pub(super) fn windows(&self, track_id: &TrackId, range: TimeRange) -> Vec<TimeRange> {
        let selected = self
            .all_tracks
            .iter()
            .chain(self.tracks.get(track_id).into_iter().flatten())
            .filter_map(|window| super::super::super::apply_ranges::intersect(*window, range))
            .collect();
        super::super::super::apply_ranges::merge(selected)
    }

    pub(super) fn add(&mut self, track_id: &TrackId, range: TimeRange) {
        let ranges = self.tracks.entry(track_id.clone()).or_default();
        ranges.push(range);
        *ranges = super::super::super::apply_ranges::merge(std::mem::take(ranges));
    }

    pub(super) fn merge(&mut self, other: &Self) -> bool {
        let before = self.clone();
        self.all_tracks.extend(other.all_tracks.iter().copied());
        self.all_tracks =
            super::super::super::apply_ranges::merge(std::mem::take(&mut self.all_tracks));
        for (track_id, ranges) in &other.tracks {
            for range in ranges {
                self.add(track_id, *range);
            }
        }
        *self != before
    }
}
