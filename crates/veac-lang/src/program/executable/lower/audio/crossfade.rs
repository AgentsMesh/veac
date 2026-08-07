use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{AudioCrossfade, AudioFadeCurve};

use super::{malformed, ExecutableLowerError};
use crate::program::executable::lower::{time, value};

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
) -> Result<Option<AudioCrossfade>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::AudioCrossfadeNone, []) => Ok(None),
        (Op::AudioCrossfadePresent, [fade_in, fade_out, curve]) => Ok(Some(AudioCrossfade {
            fade_in: time::coordinate(Some(fade_in), timebase)?,
            fade_out: time::coordinate(Some(fade_out), timebase)?,
            curve: curve_value(graph, curve)?,
        })),
        _ => Err(malformed()),
    }
}

fn curve_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<AudioFadeCurve, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::AudioFadeLinear, []) => Ok(AudioFadeCurve::Linear),
        (Op::AudioFadeEqualPower, []) => Ok(AudioFadeCurve::EqualPower),
        (Op::AudioFadeExponential, []) => Ok(AudioFadeCurve::Exponential),
        _ => Err(malformed()),
    }
}
