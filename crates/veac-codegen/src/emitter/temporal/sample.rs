use veac_plan::canonical::{
    evaluate_temporal_program, RationalTime, TemporalBindingId, TemporalClock,
    TemporalEvaluationInput, TemporalEvaluationLimits, TemporalValue,
};
use veac_plan::ResolvedRenderPlan;

use super::clock;
use super::error::TemporalBackendError;
use super::source_time;
use crate::emitter::process_owner::ProcessOwner;

pub(in crate::emitter) fn evaluate_binding(
    plan: &ResolvedRenderPlan,
    binding_id: &TemporalBindingId,
    owner: ProcessOwner<'_>,
    local_seconds: f64,
) -> Result<TemporalValue, TemporalBackendError> {
    let (binding, program) = plan.temporal_program_for(binding_id).ok_or_else(|| {
        TemporalBackendError::new(
            "TEMPORAL_BACKEND_CONTRACT",
            binding_id,
            "temporal binding or program is absent from the render plan",
        )
    })?;
    let sequence = clock::sequence(plan, owner).ok_or_else(|| {
        TemporalBackendError::new(
            "TEMPORAL_BACKEND_CONTRACT",
            binding_id,
            "temporal owner has no sequence",
        )
    })?;
    let mut inputs = Vec::with_capacity(program.inputs.len());
    for value in &binding.parameters {
        inputs.push(TemporalEvaluationInput {
            input_id: value.input_id,
            value: value.value.clone(),
        });
    }
    for value in &binding.clocks {
        inputs.push(TemporalEvaluationInput {
            input_id: value.input_id,
            value: clock_value(binding_id, owner, sequence, value.clock, local_seconds)?,
        });
    }
    inputs.sort_by_key(|value| value.input_id.get());
    evaluate_temporal_program(program, &inputs, TemporalEvaluationLimits::default()).map_err(
        |error| {
            TemporalBackendError::new("TEMPORAL_EVALUATION_FAILED", binding_id, error.to_string())
        },
    )
}

fn clock_value(
    binding_id: &TemporalBindingId,
    owner: ProcessOwner<'_>,
    sequence: &veac_plan::ResolvedSequence,
    clock: TemporalClock,
    local: f64,
) -> Result<TemporalValue, TemporalBackendError> {
    let start = match owner {
        ProcessOwner::Clip(value) => seconds(value.record_range.start),
        ProcessOwner::Apply(value) => seconds(value.record_range.start),
    };
    let sequence_seconds = start + local;
    let timescale = sequence.duration.timescale;
    Ok(match clock {
        TemporalClock::SequenceTime => TemporalValue::Time {
            value: rational(binding_id, sequence_seconds, timescale)?,
        },
        TemporalClock::ClipTime => TemporalValue::Time {
            value: rational(binding_id, local, timescale)?,
        },
        TemporalClock::SourceTime => {
            let clip = owner
                .as_clip()
                .ok_or_else(|| contract(binding_id, "source clock owner"))?;
            let value = source_time::sample(clip, local).ok_or_else(|| {
                TemporalBackendError::new(
                    "TEMPORAL_SOURCE_CLOCK_UNAVAILABLE",
                    binding_id,
                    "source-time clock requires a resolved source mapping",
                )
            })?;
            TemporalValue::Time {
                value: rational(binding_id, value, timescale)?,
            }
        }
        TemporalClock::Frame => {
            let rate = &sequence.settings.frame_rate;
            TemporalValue::Integer {
                value: (sequence_seconds * rate.numerator as f64 / f64::from(rate.denominator))
                    .floor() as i64,
            }
        }
        TemporalClock::Progress => {
            let clip = owner
                .as_clip()
                .ok_or_else(|| contract(binding_id, "progress owner"))?;
            TemporalValue::Scalar {
                value: local / seconds(clip.record_range.duration),
            }
        }
    })
}

fn rational(
    binding_id: &TemporalBindingId,
    seconds: f64,
    timescale: u32,
) -> Result<RationalTime, TemporalBackendError> {
    if !seconds.is_finite() {
        return Err(contract(binding_id, "non-finite clock value"));
    }
    let value = (seconds * f64::from(timescale)).round();
    RationalTime::new(value as i64, timescale).map_err(|_| {
        contract(
            binding_id,
            "clock value is outside the rational time contract",
        )
    })
}

fn contract(binding_id: &TemporalBindingId, message: &str) -> TemporalBackendError {
    TemporalBackendError::new("TEMPORAL_BACKEND_CONTRACT", binding_id, message)
}

fn seconds(value: RationalTime) -> f64 {
    value.value as f64 / f64::from(value.timescale)
}
