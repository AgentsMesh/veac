use std::cmp::Ordering;

use veac_ir::{RationalTime, Track};

pub(super) fn authored(tracks: &[Track], timebase: u32) -> RationalTime {
    tracks
        .iter()
        .flat_map(|track| &track.clips)
        .filter_map(|clip| clip.record_range.end().ok())
        .max_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal))
        .unwrap_or(RationalTime {
            value: 0,
            timescale: timebase,
        })
}
