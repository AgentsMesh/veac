mod linear;
mod math;

use veac_ir::{Clip, RationalTime, SourceTimeInterpolation, SourceTimeMap, TimeRange};

use math::{curve_offset, difference, difference_signed, range, sum_signed};

pub(super) fn sequence_duration(
    envelope: &veac_ir::ProjectEnvelope,
    id: &veac_ir::SequenceId,
) -> RationalTime {
    envelope
        .project
        .sequences
        .iter()
        .find(|sequence| sequence.id == *id)
        .into_iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
        .filter_map(|clip| clip.record_range.end().ok())
        .max_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(RationalTime {
            value: 0,
            timescale: envelope.project.timebase,
        })
}

pub(super) fn project(
    clip: &Clip,
    absolute: &[TimeRange],
    child_duration: RationalTime,
) -> Vec<TimeRange> {
    let Some(mapping) = &clip.source_mapping else {
        return Vec::new();
    };
    let local: Vec<_> = absolute
        .iter()
        .filter_map(|range| local_range(clip, *range))
        .collect();
    let projected = match &mapping.time_map {
        SourceTimeMap::Linear {
            source_start,
            rate,
            repeat,
            direction,
        } => linear::project(clip, &local, *source_start, *rate, *repeat, *direction),
        SourceTimeMap::Curve { segments } => curve(&local, segments, child_duration),
    };
    let Some(bounds) = RationalTime::zero(child_duration.timescale)
        .ok()
        .and_then(|start| TimeRange::new(start, child_duration).ok())
    else {
        return Vec::new();
    };
    super::super::apply_ranges::merge(
        projected
            .into_iter()
            .filter_map(|range| super::super::apply_ranges::intersect(range, bounds))
            .collect(),
    )
}

fn curve(
    local: &[TimeRange],
    segments: &[veac_ir::SourceTimeSegment],
    child_duration: RationalTime,
) -> Vec<TimeRange> {
    let Some(timebase) = segments
        .first()
        .map(|value| value.record_duration.timescale)
    else {
        return Vec::new();
    };
    let mut cursor = RationalTime::zero(timebase).ok();
    let mut output = Vec::new();
    for segment in segments {
        let Some(start) = cursor else {
            return Vec::new();
        };
        let Some(record_range) = range(start, segment.record_duration) else {
            return Vec::new();
        };
        for selected in local
            .iter()
            .filter_map(|value| super::super::apply_ranges::intersect(*value, record_range))
        {
            let projected = match segment.interpolation {
                SourceTimeInterpolation::Linear => {
                    curve_part(selected, record_range.start, segment)
                }
                SourceTimeInterpolation::Hold => hold_point(segment.source_start, child_duration),
            };
            if let Some(value) = projected {
                output.push(value);
            }
        }
        cursor = record_range.end().ok();
    }
    output
}

fn hold_point(point: RationalTime, duration: RationalTime) -> Option<TimeRange> {
    let zero = RationalTime::zero(duration.timescale).ok()?;
    let tick = RationalTime::new(1, duration.timescale).ok()?;
    let start = if point <= zero {
        zero
    } else if point >= duration {
        difference(duration, tick)?
    } else {
        point
    };
    range(start, tick)
}

fn curve_part(
    selected: TimeRange,
    record_start: RationalTime,
    segment: &veac_ir::SourceTimeSegment,
) -> Option<TimeRange> {
    let delta = difference_signed(segment.source_end, segment.source_start)?;
    let first = curve_offset(
        difference(selected.start, record_start)?,
        delta,
        segment.record_duration,
    )?;
    let last = curve_offset(
        difference(selected.end().ok()?, record_start)?,
        delta,
        segment.record_duration,
    )?;
    let first = sum_signed(segment.source_start, first)?;
    let last = sum_signed(segment.source_start, last)?;
    let (start, end) = if first <= last {
        (first, last)
    } else {
        (last, first)
    };
    super::super::time::range_between(start, end)
}

fn local_range(clip: &Clip, value: TimeRange) -> Option<TimeRange> {
    let selected = super::super::apply_ranges::intersect(clip.record_range, value)?;
    let start = difference(selected.start, clip.record_range.start)?;
    range(start, selected.duration)
}
