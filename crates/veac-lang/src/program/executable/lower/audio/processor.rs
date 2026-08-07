use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{
    AudioProcessor, AudioProcessorKind, Compressor, Gate, Limiter, LoudnessTarget, ParametricEqBand,
};

use super::{malformed, ExecutableLowerError};
use crate::program::executable::lower::{animation, id, value};

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    scope: &[&str],
) -> Result<AudioProcessor, ExecutableLowerError> {
    let (operation, values) = value::description(graph, source)?;
    let key = value::identifier(values.first())?;
    let mut path = scope.to_vec();
    path.push(key);
    Ok(AudioProcessor {
        id: id::audio_processor(&path),
        kind: kind(graph, operation, values, &path)?,
    })
}

fn kind(
    graph: &FrozenDomainGraph,
    operation: Op,
    values: &[Value],
    scope: &[&str],
) -> Result<AudioProcessorKind, ExecutableLowerError> {
    match (operation, values) {
        (Op::AudioParametricEq, [_, bands]) => Ok(AudioProcessorKind::ParametricEq {
            bands: value::list(Some(bands))?
                .iter()
                .map(|band| eq_band(graph, band, scope))
                .collect::<Result<_, _>>()?,
        }),
        (Op::AudioHighPass, [_, frequency, q, poles]) => Ok(AudioProcessorKind::HighPass {
            frequency_hz: value::finite(Some(frequency))?,
            q: value::finite(Some(q))?,
            poles: u8::try_from(value::integer(Some(poles))?).map_err(|_| malformed())?,
        }),
        (Op::AudioLowPass, [_, frequency, q, poles]) => Ok(AudioProcessorKind::LowPass {
            frequency_hz: value::finite(Some(frequency))?,
            q: value::finite(Some(q))?,
            poles: u8::try_from(value::integer(Some(poles))?).map_err(|_| malformed())?,
        }),
        (Op::AudioCompressor, [_, settings]) => {
            Ok(AudioProcessorKind::Compressor(compressor(graph, settings)?))
        }
        (Op::AudioLimiter, [_, settings]) => {
            Ok(AudioProcessorKind::Limiter(limiter(graph, settings)?))
        }
        (Op::AudioGate, [_, settings]) => Ok(AudioProcessorKind::Gate(gate(graph, settings)?)),
        (Op::AudioLoudness, [_, target]) => {
            Ok(AudioProcessorKind::Loudness(loudness(graph, target)?))
        }
        _ => Err(malformed()),
    }
}

fn eq_band(
    graph: &FrozenDomainGraph,
    source: &Value,
    scope: &[&str],
) -> Result<ParametricEqBand, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::ParametricEqBand)?;
    let [key, frequency, gain, q] = values else {
        return Err(malformed());
    };
    let key = value::identifier(Some(key))?;
    let mut path = scope.to_vec();
    path.push(key);
    Ok(ParametricEqBand {
        id: id::eq_band(&path),
        frequency_hz: value::finite(Some(frequency))?,
        gain_db: value::finite(Some(gain))?,
        q: value::finite(Some(q))?,
    })
}

fn compressor(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Compressor, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::CompressorSettings)?;
    if values.len() != 7 {
        return Err(malformed());
    }
    Ok(Compressor {
        threshold_db: value::finite(values.first())?,
        ratio: value::finite(values.get(1))?,
        attack_ms: value::finite(values.get(2))?,
        release_ms: value::finite(values.get(3))?,
        knee_db: value::finite(values.get(4))?,
        makeup_gain_db: value::finite(values.get(5))?,
        mix: animation::percent_value(graph, values.get(6))?,
    })
}

fn limiter(graph: &FrozenDomainGraph, source: &Value) -> Result<Limiter, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::LimiterSettings)?;
    let [ceiling, attack, release] = values else {
        return Err(malformed());
    };
    Ok(Limiter {
        ceiling_db: value::finite(Some(ceiling))?,
        attack_ms: value::finite(Some(attack))?,
        release_ms: value::finite(Some(release))?,
    })
}

fn gate(graph: &FrozenDomainGraph, source: &Value) -> Result<Gate, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::GateSettings)?;
    let [threshold, ratio, attack, release, range] = values else {
        return Err(malformed());
    };
    Ok(Gate {
        threshold_db: value::finite(Some(threshold))?,
        ratio: value::finite(Some(ratio))?,
        attack_ms: value::finite(Some(attack))?,
        release_ms: value::finite(Some(release))?,
        range_db: value::finite(Some(range))?,
    })
}

fn loudness(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<LoudnessTarget, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::LoudnessTarget)?;
    let [integrated, peak, range] = values else {
        return Err(malformed());
    };
    Ok(LoudnessTarget {
        integrated_lufs: value::finite(Some(integrated))?,
        true_peak_dbtp: value::finite(Some(peak))?,
        loudness_range_lu: value::finite(Some(range))?,
    })
}
