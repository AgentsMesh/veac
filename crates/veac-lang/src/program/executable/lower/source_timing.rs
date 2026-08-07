use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::{ExactNumber, Value};
use crate::program::DomainOperationId as Op;
use veac_ir::{
    FrameSynthesisPolicy, PlaybackDirection, Rational, SourceMapping, SourceOutOfRangePolicy,
    SourceTimeInterpolation, SourceTimeMap, SourceTimeSegment,
};

use super::error::ExecutableLowerError;
use super::{time, value};

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    timed_source: bool,
) -> Result<Option<SourceMapping>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::SourceTimingNative, []) if timed_source => Ok(Some(SourceMapping::linear(
            malformed_result(veac_ir::RationalTime::zero(timebase))?,
            malformed_result(Rational::new(1, 1))?,
        ))),
        (Op::SourceTimingNative, []) => Ok(None),
        (Op::SourceTimingMapped, [mapping]) => Ok(Some(mapping_value(graph, mapping, timebase)?)),
        _ => Err(malformed()),
    }
}

fn mapping_value(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
) -> Result<SourceMapping, ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::SourceMapping)?;
    if operands.len() != 3 {
        return Err(malformed());
    }
    Ok(SourceMapping {
        time_map: time_map(graph, &operands[0], timebase)?,
        frame_synthesis: synthesis(graph, &operands[1])?,
        out_of_range: out_of_range(graph, &operands[2])?,
    })
}

fn time_map(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
) -> Result<SourceTimeMap, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::SourceTimeLinear, [start, rate, repeat, direction]) => Ok(SourceTimeMap::Linear {
            source_start: time::coordinate(Some(start), timebase)?,
            rate: rational(value::exact(Some(rate))?)?,
            repeat: malformed_result(u32::try_from(value::integer(Some(repeat))?))?,
            direction: direction_value(graph, direction)?,
        }),
        (Op::SourceTimeCurve, [segments]) => Ok(SourceTimeMap::Curve {
            segments: value::list(Some(segments))?
                .iter()
                .map(|segment| segment_value(graph, segment, timebase))
                .collect::<Result<_, _>>()?,
        }),
        _ => Err(malformed()),
    }
}

fn segment_value(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
) -> Result<SourceTimeSegment, ExecutableLowerError> {
    let operands = value::description_operands(graph, source, Op::SourceTimeSegment)?;
    if operands.len() != 4 {
        return Err(malformed());
    }
    Ok(SourceTimeSegment {
        record_duration: time::coordinate(operands.first(), timebase)?,
        source_start: time::coordinate(operands.get(1), timebase)?,
        source_end: time::coordinate(operands.get(2), timebase)?,
        interpolation: interpolation(graph, &operands[3])?,
    })
}

fn rational(value: ExactNumber) -> Result<Rational, ExecutableLowerError> {
    let numerator = malformed_result(i64::try_from(value.numerator()))?;
    let denominator = malformed_result(u32::try_from(value.denominator()))?;
    malformed_result(Rational::new(numerator, denominator))
}

fn direction_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<PlaybackDirection, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::PlaybackForward, []) => Ok(PlaybackDirection::Forward),
        (Op::PlaybackReverse, []) => Ok(PlaybackDirection::Reverse),
        _ => Err(malformed()),
    }
}

fn synthesis(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<FrameSynthesisPolicy, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::FrameNearest, []) => Ok(FrameSynthesisPolicy::Nearest),
        (Op::FrameBlend, []) => Ok(FrameSynthesisPolicy::Blend),
        (Op::FrameMotionCompensated, []) => Ok(FrameSynthesisPolicy::MotionCompensated),
        _ => Err(malformed()),
    }
}

fn out_of_range(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<SourceOutOfRangePolicy, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::OutOfRangeStrict, []) => Ok(SourceOutOfRangePolicy::Strict),
        (Op::OutOfRangeHoldFirst, []) => Ok(SourceOutOfRangePolicy::HoldFirst),
        (Op::OutOfRangeHoldLast, []) => Ok(SourceOutOfRangePolicy::HoldLast),
        (Op::OutOfRangeHoldBoth, []) => Ok(SourceOutOfRangePolicy::HoldBoth),
        _ => Err(malformed()),
    }
}

fn interpolation(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<SourceTimeInterpolation, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::SegmentLinear, []) => Ok(SourceTimeInterpolation::Linear),
        (Op::SegmentHold, []) => Ok(SourceTimeInterpolation::Hold),
        _ => Err(malformed()),
    }
}

fn malformed() -> ExecutableLowerError {
    value::graph("an executable item has invalid typed source timing")
}

fn malformed_result<T, E>(result: Result<T, E>) -> Result<T, ExecutableLowerError> {
    match result {
        Ok(value) => Ok(value),
        Err(_) => Err(malformed()),
    }
}
