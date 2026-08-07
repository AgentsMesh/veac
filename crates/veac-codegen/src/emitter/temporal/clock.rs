use veac_plan::canonical::{TemporalClock, TemporalClockBinding};
use veac_plan::{ResolvedRenderPlan, ResolvedSequence};

use super::budget::Budget;
use super::error::TemporalBackendError;
use super::expression;
use super::source_time;
use super::value::{number, CompiledValue, Expression};
use crate::emitter::process_owner::ProcessOwner;

pub(super) fn compile(
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    binding: &TemporalClockBinding,
    local_clock: &str,
    budget: &mut Budget<'_>,
) -> Result<CompiledValue, TemporalBackendError> {
    let sequence =
        sequence(plan, owner).ok_or_else(|| budget.contract("clock owner has no sequence"))?;
    let local = budget.literal(local_clock.to_owned())?;
    Ok(match binding.clock {
        TemporalClock::SequenceTime => CompiledValue::Time(sequence_time(owner, &local, budget)?),
        TemporalClock::ClipTime => {
            require_clip(owner, budget)?;
            CompiledValue::Time(local)
        }
        TemporalClock::SourceTime => {
            let clip = require_clip(owner, budget)?;
            CompiledValue::Time(source_time::compile(clip, &local, budget)?)
        }
        TemporalClock::Frame => {
            let clock = sequence_time(owner, &local, budget)?;
            let numerator = number(budget, sequence.settings.frame_rate.numerator as f64)?;
            let denominator = number(budget, f64::from(sequence.settings.frame_rate.denominator))?;
            let scaled = expression::infix(budget, &clock, "*", &numerator)?;
            let scaled = expression::infix(budget, &scaled, "/", &denominator)?;
            CompiledValue::Integer(expression::call1(budget, "floor", &scaled)?)
        }
        TemporalClock::Progress => {
            let clip = require_clip(owner, budget)?;
            let duration = number(budget, seconds(clip.record_range.duration))?;
            CompiledValue::Scalar(expression::infix(budget, &local, "/", &duration)?)
        }
    })
}

pub(super) fn sequence<'a>(
    plan: &'a ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
) -> Option<&'a ResolvedSequence> {
    plan.sequences.iter().find(|sequence| match owner {
        ProcessOwner::Clip(clip) => sequence
            .tracks
            .iter()
            .flat_map(|track| &track.clips)
            .any(|value| value.id == clip.id),
        ProcessOwner::Apply(apply) => sequence.applies.iter().any(|value| value.id == apply.id),
    })
}

pub(super) fn sequence_time(
    owner: ProcessOwner<'_>,
    local: &Expression,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let start = match owner {
        ProcessOwner::Clip(clip) => clip.record_range.start,
        ProcessOwner::Apply(apply) => apply.record_range.start,
    };
    let start = number(budget, seconds(start))?;
    expression::infix(budget, local, "+", &start)
}

fn require_clip<'a>(
    owner: ProcessOwner<'a>,
    budget: &Budget<'_>,
) -> Result<&'a veac_plan::ResolvedClip, TemporalBackendError> {
    owner
        .as_clip()
        .ok_or_else(|| budget.contract("item clock is bound to a sequence process"))
}

fn seconds(value: veac_plan::canonical::RationalTime) -> f64 {
    value.value as f64 / f64::from(value.timescale)
}
