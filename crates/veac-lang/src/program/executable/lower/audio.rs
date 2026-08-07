use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{AudioProperties, PitchPolicy};

use super::error::ExecutableLowerError;
use super::{animation, value};

mod crossfade;
mod processor;

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    scope: &[&str],
) -> Result<AudioProperties, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::AudioStyle)?;
    if values.len() != 5 {
        return Err(malformed());
    }
    let playback = value::description_operands(graph, &values[2], Op::AudioPlayback)?;
    if playback.len() != 3 {
        return Err(malformed());
    }
    Ok(AudioProperties {
        gain: animation::scalar(graph, &values[0], timebase, &channel(scope, "audio.gain"))?,
        pan: animation::scalar(graph, &values[1], timebase, &channel(scope, "audio.pan"))?,
        muted: value::boolean(playback.first())?,
        normalize: value::boolean(playback.get(1))?,
        pitch_policy: pitch(graph, &playback[2])?,
        processors: value::list(values.get(3))?
            .iter()
            .map(|source| processor::lower(graph, source, scope))
            .collect::<Result<_, _>>()?,
        crossfade: crossfade::lower(graph, &values[4], timebase)?,
    })
}

fn pitch(graph: &FrozenDomainGraph, source: &Value) -> Result<PitchPolicy, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::PitchPreserve, []) => Ok(PitchPolicy::Preserve),
        (Op::PitchFollowSpeed, []) => Ok(PitchPolicy::FollowSpeed),
        _ => Err(malformed()),
    }
}

fn channel<'a>(scope: &[&'a str], name: &'a str) -> Vec<&'a str> {
    let mut result = scope.to_vec();
    result.push(name);
    result
}

fn malformed() -> ExecutableLowerError {
    value::graph("an executable audio style has invalid typed topology")
}
