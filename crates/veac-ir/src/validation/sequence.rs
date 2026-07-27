use std::collections::BTreeSet;

use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn sequence(&mut self, sequence: &Sequence, timebase: u32) {
        let path = format!("/project/sequences/{}", sequence.id);
        if sequence.name.trim().is_empty() {
            self.push(
                "SEQUENCE_NAME",
                Some(sequence.id.to_string()),
                format!("{path}/name"),
                "sequence name must be non-empty",
                None,
            );
        }
        self.metadata(
            &sequence.metadata,
            &format!("{path}/metadata"),
            sequence.id.as_str(),
        );
        if !render_geometry_valid(
            sequence.settings.width,
            sequence.settings.height,
            sequence.settings.frame_rate,
        ) || !ffmpeg_sample_rate_valid(sequence.settings.sample_rate)
        {
            self.push(
                "SEQUENCE_SETTINGS",
                Some(sequence.id.to_string()),
                format!("{path}/settings"),
                "sequence settings exceed the default untrusted-plan render budget",
                None,
            );
        }
        self.tracks(sequence, timebase, &path);
        self.applies(sequence, timebase, &path);
    }

    fn tracks(&mut self, sequence: &Sequence, timebase: u32, path: &str) {
        let mut orders = BTreeSet::new();
        for track in &sequence.tracks {
            let track_path = format!("{path}/tracks/{}", track.id);
            self.check_id(track.id.is_valid(), track.id.as_str(), &track_path);
            if !self.track_ids.insert(track.id.to_string()) {
                self.duplicate("DUPLICATE_TRACK_ID", track.id.as_str(), &track_path);
            }
            if !orders.insert(track.order) {
                self.push(
                    "DUPLICATE_TRACK_ORDER",
                    Some(track.id.to_string()),
                    format!("{track_path}/order"),
                    "track order must be unique",
                    None,
                );
            }
            self.track(track, timebase, sequence.settings.sample_rate, &track_path);
        }
    }

    fn track(&mut self, track: &Track, timebase: u32, sample_rate: u32, path: &str) {
        let mut previous_start: Option<RationalTime> = None;
        let mut magnetic_end = RationalTime {
            value: 0,
            timescale: timebase,
        };
        for clip in &track.clips {
            let clip_path = format!("{path}/clips/{}", clip.id);
            self.check_id(clip.id.is_valid(), clip.id.as_str(), &clip_path);
            if !self.item_ids.insert(clip.id.to_string()) {
                self.duplicate("DUPLICATE_ITEM_ID", clip.id.as_str(), &clip_path);
            }
            self.time_range(
                clip.record_range,
                timebase,
                "RECORD_RANGE",
                &format!("{clip_path}/record_range"),
                clip.id.as_str(),
            );
            if previous_start.is_some_and(|start| clip.record_range.start < start) {
                self.value_error("TRACK_ORDER", &clip_path, clip.id.as_str());
            }
            previous_start = Some(clip.record_range.start);
            if track.placement_mode == PlacementMode::Magnetic
                && clip.record_range.start != magnetic_end
            {
                self.value_error("MAGNETIC_CONTINUITY", &clip_path, clip.id.as_str());
            }
            if let Ok(end) = clip.record_range.end() {
                magnetic_end = end;
            }
            self.clip(clip, track.kind, timebase, sample_rate, &clip_path);
        }
    }
}
