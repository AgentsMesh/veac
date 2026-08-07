use std::collections::BTreeSet;

use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn sequence(&mut self, sequence: &Sequence, timebase: u32, relations: &[Relation]) {
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
        self.sequence_authorship(sequence, relations, &path);
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
        self.tracks(sequence, timebase, relations, &path);
        self.applies(sequence, timebase, &path);
    }

    fn tracks(&mut self, sequence: &Sequence, timebase: u32, relations: &[Relation], path: &str) {
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
            self.track(
                sequence,
                track,
                timebase,
                sequence.settings.sample_rate,
                relations,
                &track_path,
            );
        }
    }

    fn track(
        &mut self,
        sequence: &Sequence,
        track: &Track,
        timebase: u32,
        sample_rate: u32,
        relations: &[Relation],
        path: &str,
    ) {
        let mut previous_start: Option<RationalTime> = None;
        let mut previous_clip: Option<&Clip> = None;
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
            let transition_overlap = previous_clip.is_some_and(|previous| {
                clip.record_range.start < magnetic_end
                    && has_transition(relations, &sequence.id, &previous.id, &clip.id)
            });
            if track.placement_mode == PlacementMode::Magnetic
                && clip.record_range.start != magnetic_end
                && !transition_overlap
            {
                self.value_error("MAGNETIC_CONTINUITY", &clip_path, clip.id.as_str());
            }
            if let Ok(end) = clip.record_range.end() {
                magnetic_end = end;
            }
            previous_clip = Some(clip);
            self.clip(clip, track.kind, timebase, sample_rate, &clip_path);
        }
    }
}

fn has_transition(
    relations: &[Relation],
    sequence: &SequenceId,
    from: &ItemId,
    to: &ItemId,
) -> bool {
    relations.iter().any(|relation| {
        if relation.sequence_id != *sequence {
            return false;
        }
        let RelationKind::Transition {
            from: left,
            to: right,
            ..
        } = &relation.kind
        else {
            return false;
        };
        left.item_id() == Some(from) && right.item_id() == Some(to)
    })
}
