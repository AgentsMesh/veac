use std::cmp::Ordering;

use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn source_mapping(&mut self, clip: &Clip, timebase: u32, path: &str) {
        let Some(mapping) = &clip.source_mapping else {
            return;
        };
        let map_path = format!("{path}/source_mapping/time_map");
        match &mapping.time_map {
            SourceTimeMap::Linear {
                source_start,
                rate,
                repeat,
                ..
            } => {
                self.source_coordinate(
                    *source_start,
                    timebase,
                    mapping.out_of_range,
                    &format!("{map_path}/source_start"),
                    clip.id.as_str(),
                );
                if !rate.is_positive() {
                    self.value_error("SOURCE_RATE", &map_path, clip.id.as_str());
                }
                if !(1..=MAX_SOURCE_REPEAT).contains(repeat) {
                    self.value_error("SOURCE_REPEAT", &map_path, clip.id.as_str());
                }
            }
            SourceTimeMap::Curve { segments } => {
                self.source_curve(clip, segments, mapping.out_of_range, timebase, &map_path);
            }
        }
    }

    fn source_curve(
        &mut self,
        clip: &Clip,
        segments: &[SourceTimeSegment],
        outside: SourceOutOfRangePolicy,
        timebase: u32,
        path: &str,
    ) {
        if segments.is_empty() {
            self.value_error("SOURCE_TIME_MAP_EMPTY", path, clip.id.as_str());
            return;
        }
        let mut total = RationalTime {
            value: 0,
            timescale: timebase,
        };
        let mut previous_end = None;
        let mut direction = None;
        for (index, segment) in segments.iter().enumerate() {
            let item_path = format!("{path}/segments/{index}");
            self.time(
                segment.record_duration,
                timebase,
                true,
                "SOURCE_TIME_MAP",
                &item_path,
                clip.id.as_str(),
            );
            self.source_coordinate(
                segment.source_start,
                timebase,
                outside,
                &item_path,
                clip.id.as_str(),
            );
            self.source_coordinate(
                segment.source_end,
                timebase,
                outside,
                &item_path,
                clip.id.as_str(),
            );
            if previous_end.is_some_and(|end| end != segment.source_start) {
                self.value_error("SOURCE_TIME_MAP_CONTINUITY", &item_path, clip.id.as_str());
            }
            self.source_segment_shape(segment, &item_path, clip.id.as_str(), &mut direction);
            previous_end = Some(segment.source_end);
            match total.checked_add(segment.record_duration) {
                Ok(value) => total = value,
                Err(_) => self.value_error("SOURCE_TIME_MAP_DURATION", path, clip.id.as_str()),
            }
        }
        if total != clip.record_range.duration {
            self.value_error("SOURCE_TIME_MAP_DURATION", path, clip.id.as_str());
        }
    }

    fn source_coordinate(
        &mut self,
        value: RationalTime,
        timebase: u32,
        outside: SourceOutOfRangePolicy,
        path: &str,
        id: &str,
    ) {
        if value.timescale != timebase || value.value < 0 && !outside.allows_before() {
            self.value_error("SOURCE_TIME", path, id);
        }
    }

    fn source_segment_shape(
        &mut self,
        segment: &SourceTimeSegment,
        path: &str,
        id: &str,
        direction: &mut Option<Ordering>,
    ) {
        let ordering = segment.source_end.partial_cmp(&segment.source_start);
        let shape_valid = matches!(
            (segment.interpolation, ordering),
            (SourceTimeInterpolation::Hold, Some(Ordering::Equal))
                | (
                    SourceTimeInterpolation::Linear,
                    Some(Ordering::Less | Ordering::Greater)
                )
        );
        if !shape_valid {
            self.value_error("SOURCE_TIME_INTERPOLATION", path, id);
        }
        if let Some(ordering @ (Ordering::Less | Ordering::Greater)) = ordering {
            if direction.is_some_and(|known| known != ordering) {
                self.value_error("SOURCE_TIME_MAP_MONOTONIC", path, id);
            } else {
                *direction = Some(ordering);
            }
        }
    }
}
